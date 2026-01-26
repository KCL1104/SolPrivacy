// Allow deprecated spl_token_2022::instruction - migration to spl_token_2022_interface planned
#![allow(deprecated)]

use crate::commands::confidential::{ConfidentialAction, ConfidentialCommand};
use crate::config::AppConfig;
use crate::error::{Result, SolPrivacyError};
use crate::error_decoder::decode_transaction_error;
use crate::validation::{ui_amount_to_amount, validate_pubkey};
use clap::Args;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022::{id as token_2022_program_id, instruction as token_instruction};

/// Transfer tokens (supports confidential mode)
#[derive(Args)]
pub struct TransferCommand {
    /// Mint address of the token
    #[arg(short, long)]
    pub mint: String,

    /// Recipient address
    #[arg(short, long)]
    pub to: String,

    /// Amount to transfer (in token units, e.g., 100.5)
    #[arg(short, long)]
    pub amount: f64,

    /// Path to sender keypair
    #[arg(short, long, env = "SOLANA_KEYPAIR")]
    pub keypair: Option<String>,

    /// Use confidential transfer (encrypted amounts)
    #[arg(long)]
    pub confidential: bool,

    /// Path to ElGamal keypair (required for confidential transfer)
    #[arg(long)]
    pub elgamal_keypair: Option<String>,

    /// Dry run - simulate without sending
    #[arg(long)]
    pub dry_run: bool,
}

impl TransferCommand {
    pub async fn run(&self) -> Result<()> {
        println!("{} Token Transfer", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;
        let rpc_url = config.get_rpc_url();

        // Parse and validate addresses
        let mint_pubkey = validate_pubkey(&self.mint)?;
        let recipient = validate_pubkey(&self.to)?;

        // Validate amount (using 9 decimals as default for SPL tokens)
        let _amount_raw = ui_amount_to_amount(self.amount, 9)?;

        println!("  {}:", "Transfer Details".bright_white());
        println!("  ├─ Mint: {}...", &self.mint[..16.min(self.mint.len())]);
        println!("  ├─ To: {}...", &self.to[..16.min(self.to.len())]);
        println!("  ├─ Amount: {}", format!("{}", self.amount).bright_cyan());
        println!(
            "  ├─ Mode: {}",
            if self.confidential {
                "Confidential (encrypted)".bright_green()
            } else {
                "Public".bright_yellow()
            }
        );
        println!("  └─ Network: {}", config.network);
        println!();

        if self.confidential {
            println!("{} Confidential Transfer Mode", "→".bright_cyan());
            println!("  Redirecting to confidential module...");
            println!();

            let cmd = ConfidentialCommand {
                action: ConfidentialAction::Transfer {
                    mint: self.mint.clone(),
                    to: self.to.clone(),
                    amount: self.amount,
                    elgamal_keypair: self.elgamal_keypair.clone(),
                    keypair: self.keypair.clone(),
                    dry_run: self.dry_run,
                },
            };
            return cmd.run().await;
        }

        // Standard (public) transfer
        if self.dry_run {
            println!("{} Dry run - no transaction sent", "ℹ".bright_blue());
            println!();
            println!("  Would transfer {} tokens", self.amount);
            println!("  From: (your account)");
            println!("  To: {}", recipient);
            return Ok(());
        }

        // Load keypair
        let keypair_path = match &self.keypair {
            Some(p) => p.clone(),
            None => {
                let default = dirs::home_dir()
                    .map(|h| h.join(".config/solana/id.json"))
                    .and_then(|p| p.to_str().map(|s| s.to_string()));

                match default {
                    Some(p) if std::path::Path::new(&p).exists() => p,
                    _ => {
                        println!("{} Keypair required!", "✗".bright_red());
                        return Ok(());
                    }
                }
            }
        };

        let payer = read_keypair_file(&keypair_path)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to read keypair: {}", e)))?;

        println!("{} Connecting...", "→".bright_cyan());
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        // Get mint info to determine decimals
        let mint_account = client
            .get_account(&mint_pubkey)
            .map_err(|e| SolPrivacyError::Other(format!("Failed to get mint: {}", e)))?;

        // Parse decimals from mint data (offset 44 for standard mint)
        let decimals = if mint_account.data.len() >= 45 {
            mint_account.data[44]
        } else {
            9 // default
        };

        // Calculate raw amount
        let raw_amount = (self.amount * 10f64.powi(decimals as i32)) as u64;

        // Get source and destination ATAs
        let source_ata = get_associated_token_address_with_program_id(
            &payer.pubkey(),
            &mint_pubkey,
            &token_2022_program_id(),
        );

        let dest_ata = get_associated_token_address_with_program_id(
            &recipient,
            &mint_pubkey,
            &token_2022_program_id(),
        );

        println!("  Source: {}", source_ata);
        println!("  Destination: {}", dest_ata);
        println!();

        // Build instructions
        let mut instructions = vec![];

        // Check if destination ATA exists, if not create it
        if client.get_account(&dest_ata).is_err() {
            println!("{} Creating destination account...", "→".bright_cyan());
            instructions.push(create_associated_token_account(
                &payer.pubkey(),
                &recipient,
                &mint_pubkey,
                &token_2022_program_id(),
            ));
        }

        // Transfer instruction
        instructions.push(
            token_instruction::transfer_checked(
                &token_2022_program_id(),
                &source_ata,
                &mint_pubkey,
                &dest_ata,
                &payer.pubkey(),
                &[],
                raw_amount,
                decimals,
            )
            .map_err(|e| SolPrivacyError::Other(format!("Failed to create transfer: {:?}", e)))?,
        );

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message("Sending transaction...");

        let blockhash = client
            .get_latest_blockhash()
            .map_err(|e| SolPrivacyError::Other(format!("Failed to get blockhash: {}", e)))?;

        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer],
            blockhash,
        );

        let signature = client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| SolPrivacyError::Other(decode_transaction_error(&e.to_string())))?;

        pb.finish_and_clear();

        println!("{} Transfer complete!", "✓".bright_green());
        println!();
        println!("  {}:", "Details".bright_white());
        println!("  ├─ Amount: {} tokens", self.amount);
        println!("  ├─ To: {}", recipient);
        println!("  └─ Signature: {}", signature);
        println!();

        let explorer = match config.network.as_str() {
            "mainnet" => format!("https://solscan.io/tx/{}", signature),
            _ => format!("https://solscan.io/tx/{}?cluster=devnet", signature),
        };
        println!("  View: {}", explorer);

        Ok(())
    }
}
