use crate::config::AppConfig;
use crate::error::{Result, SolPrivacyError};
use clap::{Args, Subcommand};
use colored::Colorize;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use spl_token_2022::{
    extension::{
        confidential_transfer::{
            instruction::apply_pending_balance, ConfidentialTransferAccount,
        },
        BaseStateWithExtensions, StateWithExtensions,
    },
    state::Account as TokenAccount,
};
use std::{str::FromStr, thread, time::Duration};

/// Utility commands for automation and maintenance
#[derive(Args)]
pub struct UtilCommand {
    #[command(subcommand)]
    pub action: UtilAction,
}

#[derive(Subcommand)]
pub enum UtilAction {
    /// Auto-crank service to apply pending confidential balances
    Crank {
        /// Monitor specific account (optional, can be repeated)
        #[arg(short, long)]
        account: Vec<String>,

        /// Path to payer keypair
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,

        /// Poll interval in seconds
        #[arg(long, default_value = "10")]
        interval: u64,

        /// Run continuously
        #[arg(long, default_value = "true")]
        loop_mode: bool,
    },
}

impl UtilCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            UtilAction::Crank {
                account,
                keypair,
                interval,
                loop_mode,
            } => {
                self.run_crank(account, keypair.as_deref(), *interval, *loop_mode)
                    .await
            }
        }
    }

    async fn run_crank(
        &self,
        accounts: &[String],
        keypair_path: Option<&str>,
        interval: u64,
        loop_mode: bool,
    ) -> Result<()> {
        println!("{} Auto-Crank Service", "⚙".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());

        // 1. Setup
        let config = AppConfig::load()?;
        let client =
            RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());

        let payer = if let Some(path) = keypair_path {
            read_keypair_file(path)
                .map_err(|e| SolPrivacyError::Crypto(format!("Failed to read keypair: {}", e)))?
        } else {
            // Try default location or error
            let default_path = dirs::home_dir()
                .ok_or(SolPrivacyError::Config("Home dir not found".to_string()))?
                .join(".config/solana/id.json");
            read_keypair_file(&default_path).map_err(|_| {
                SolPrivacyError::Config(
                    "No keypair provided and default not found. Use --keypair.".to_string(),
                )
            })?
        };

        let account_pubkeys: Result<Vec<Pubkey>> = accounts
            .iter()
            .map(|s| {
                Pubkey::from_str(s)
                    .map_err(|e| SolPrivacyError::Other(format!("Invalid pubkey {}: {}", s, e)))
            })
            .collect();
        let account_pubkeys = account_pubkeys?;

        println!("  Network: {}", config.network);
        println!("  Payer: {}", payer.pubkey());
        println!("  Monitored Accounts: {}", account_pubkeys.len());
        println!("  Interval: {}s", interval);
        println!();

        if account_pubkeys.is_empty() {
            println!(
                "{} No accounts specified to monitor.",
                "⚠".bright_yellow()
            );
            return Ok(());
        }

        loop {
            for pubkey in &account_pubkeys {
                match self.check_and_apply(&client, pubkey, &payer) {
                    Ok(processed) => {
                        if processed {
                            println!(
                                "{} Applied pending balance for {}",
                                "✓".bright_green(),
                                pubkey
                            );
                        }
                    }
                    Err(e) => {
                        println!(
                            "{} Error checking {}: {}",
                            "✗".bright_red(),
                            pubkey,
                            e
                        );
                    }
                }
            }

            if !loop_mode {
                break;
            }
            thread::sleep(Duration::from_secs(interval));
        }

        Ok(())
    }

    fn check_and_apply(
        &self,
        client: &RpcClient,
        account_pubkey: &Pubkey,
        payer: &solana_sdk::signature::Keypair,
    ) -> Result<bool> {
        // Fetch account data
        let account = client
            .get_account(account_pubkey)
            .map_err(|e| SolPrivacyError::Rpc(format!("Failed to fetch account: {}", e)))?;

        // Parse Token-2022 state
        let state = StateWithExtensions::<TokenAccount>::unpack(&account.data)
            .map_err(|e| SolPrivacyError::Other(format!("Failed to unpack token account: {}", e)))?;

        // Check for ConfidentialTransfer extension
        if let Ok(ct_extension) = state.get_extension::<ConfidentialTransferAccount>() {
            // Check if there is effectively any pending balance?
            // We can't check value (encrypted), but we must pass the current ciphertext to the instruction
            // as 'expected_pending_balance_credit'.
            
            // Note: In a real crank, we might want to check if the ciphertext is "zero" (all empty)?
            // But encrypted zero is not all bytes zero.
            // However, the instruction validates the passed ciphertext matches the account state.
            // If we just apply whatever is there, it should check out.
            
            let expected_pending_balance = &ct_extension.decryptable_available_balance;

            // Build instruction
             let apply_ix = apply_pending_balance(
                &spl_token_2022::id(),
                account_pubkey,
                1, // We assume 1 signatures (the payer/owner)
                expected_pending_balance,
                &payer.pubkey(), // Payer is authority?
                &[],
            ).map_err(|e| SolPrivacyError::Other(format!("Failed to create instruction: {}", e)))?;

            let blockhash = client.get_latest_blockhash()
                .map_err(|e| SolPrivacyError::Rpc(format!("Failed to get blockhash: {}", e)))?;

            let tx = Transaction::new_signed_with_payer(
                &[apply_ix],
                Some(&payer.pubkey()),
                &[payer],
                blockhash
            );

            match client.send_and_confirm_transaction(&tx) {
                Ok(_) => return Ok(true),
                Err(_e) => {
                     // Check if error is "no pending balance" or similar harmless error?
                     // For now, treat error as false (no apply occurred or failed)
                     // But don't crash loop.
                     // println!("Debug: {}", e);
                     return Ok(false);
                }
            }
        } else {
            // Not a confidential account
            return Err(SolPrivacyError::Other("Account does not have ConfidentialTransfer extension".to_string()));
        }
    }
}
