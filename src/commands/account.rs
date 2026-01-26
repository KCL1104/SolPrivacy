use crate::config::AppConfig;
use crate::error::{Result, SolPrivacyError};
use crate::validation::validate_pubkey;
use clap::{Args, Subcommand};
use colored::Colorize;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id, instruction::create_associated_token_account,
};
use spl_token_2022::id as token_2022_program_id;

/// Manage token accounts
#[derive(Args)]
pub struct AccountCommand {
    #[command(subcommand)]
    pub action: AccountAction,
}

#[derive(Subcommand)]
pub enum AccountAction {
    /// Create a new token account (ATA)
    Create {
        /// Mint address
        #[arg(short, long)]
        mint: String,

        /// Owner address (default: your wallet)
        #[arg(short, long)]
        owner: Option<String>,

        /// Path to payer keypair
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,
    },

    /// Show token account details
    Info {
        /// Token account address
        #[arg()]
        account: String,
    },

    /// List token accounts for a wallet
    List {
        /// Wallet address (default: your wallet)
        #[arg()]
        wallet: Option<String>,

        /// Path to keypair (for default wallet)
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,
    },

    /// Configure account for confidential transfers
    ConfigureConfidential {
        /// Token account address (ATA)
        #[arg(short, long)]
        account: String,

        /// Mint address
        #[arg(short, long)]
        mint: String,
    },
}

impl AccountCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            AccountAction::Create {
                mint,
                owner,
                keypair,
            } => {
                self.create_account(mint, owner.as_deref(), keypair.as_deref())
                    .await
            }
            AccountAction::Info { account } => self.show_info(account).await,
            AccountAction::List { wallet, keypair } => {
                self.list_accounts(wallet.as_deref(), keypair.as_deref())
                    .await
            }
            AccountAction::ConfigureConfidential { account, mint } => {
                self.configure_confidential(account, mint).await
            }
        }
    }

    async fn configure_confidential(&self, account: &str, mint: &str) -> Result<()> {
        println!(
            "{} Configure Account for Confidential Transfers",
            "→".bright_cyan()
        );
        println!("{}", "─".repeat(50).bright_black());
        println!();

        println!("  {}:", "Account Details".bright_white());
        println!("  ├─ Account: {}...", &account[..16.min(account.len())]);
        println!("  └─ Mint: {}...", &mint[..16.min(mint.len())]);
        println!();

        println!(
            "  {}:",
            "Redirecting to Confidential Command".bright_white()
        );
        println!();
        println!("  For full confidential transfer configuration, use:");
        println!();
        println!("    solprivacy confidential configure \\");
        println!("      --account {} \\", account);
        println!("      --mint {} \\", mint);
        println!("      --elgamal-keypair <PATH_TO_ELGAMAL_KEY>");
        println!();
        println!("  {}:", "Prerequisites".bright_white());
        println!("    1. Generate ElGamal keypair:");
        println!("       solprivacy keygen elgamal -o elgamal.json");
        println!();
        println!("    2. Ensure account has the ConfidentialTransfer extension");
        println!("       (created during mint with confidential extension)");
        println!();
        println!("  {}:", "Full Workflow".bright_white());
        println!("    solprivacy confidential workflow");

        Ok(())
    }

    async fn create_account(
        &self,
        mint: &str,
        owner: Option<&str>,
        keypair_path: Option<&str>,
    ) -> Result<()> {
        println!("{} Create Token Account", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;
        let rpc_url = config.get_rpc_url();

        let mint_pubkey = validate_pubkey(mint)?;

        // Load keypair
        let keypair_path = match keypair_path {
            Some(p) => p.to_string(),
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

        let owner_pubkey = match owner {
            Some(o) => validate_pubkey(o)?,
            None => payer.pubkey(),
        };

        println!("  {}:", "Account Details".bright_white());
        println!("  ├─ Mint: {}...", &mint[..16.min(mint.len())]);
        println!("  ├─ Owner: {}", owner_pubkey);
        println!("  └─ Network: {}", config.network);
        println!();

        // Calculate ATA
        let ata = get_associated_token_address_with_program_id(
            &owner_pubkey,
            &mint_pubkey,
            &token_2022_program_id(),
        );

        println!("{} Connecting...", "→".bright_cyan());
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        // Check if account already exists
        if client.get_account(&ata).is_ok() {
            println!("{} Account already exists!", "✓".bright_green());
            println!();
            println!("  Address: {}", ata);
            return Ok(());
        }

        // Create ATA
        println!("{} Creating account...", "→".bright_cyan());

        let instruction = create_associated_token_account(
            &payer.pubkey(),
            &owner_pubkey,
            &mint_pubkey,
            &token_2022_program_id(),
        );

        let blockhash = client
            .get_latest_blockhash()
            .map_err(|e| SolPrivacyError::Other(format!("Failed to get blockhash: {}", e)))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            blockhash,
        );

        let signature = client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| SolPrivacyError::Other(format!("Failed to create account: {}", e)))?;

        println!("{} Account created!", "✓".bright_green());
        println!();
        println!("  {}:", "Details".bright_white());
        println!("  ├─ Address: {}", ata.to_string().bright_cyan());
        println!("  ├─ Owner: {}", owner_pubkey);
        println!("  └─ Transaction: {}", signature);
        println!();

        println!("  {}:", "Next Steps".bright_white());
        println!("    Receive tokens:");
        println!(
            "      solprivacy transfer --mint {} --to {} --amount 100",
            mint, owner_pubkey
        );

        Ok(())
    }

    async fn show_info(&self, account: &str) -> Result<()> {
        println!("{} Token Account Info", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;
        let client =
            RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());

        let account_pubkey = validate_pubkey(account)?;

        match client.get_account(&account_pubkey) {
            Ok(acc) => {
                let is_token_2022 =
                    acc.owner.to_string() == "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

                println!("  {}:", "Account Info".bright_white());
                println!("  ├─ Address: {}", account_pubkey);
                println!(
                    "  ├─ Program: {}",
                    if is_token_2022 {
                        "Token-2022"
                    } else {
                        "Unknown"
                    }
                );
                println!("  ├─ Lamports: {:.6} SOL", acc.lamports as f64 / 1e9);
                println!("  └─ Data Size: {} bytes", acc.data.len());

                // Try to get balance
                if let Ok(balance) = client.get_token_account_balance(&account_pubkey) {
                    println!();
                    println!("  {}:", "Token Balance".bright_white());
                    println!("  ├─ Amount: {}", balance.ui_amount_string.bright_cyan());
                    println!("  └─ Decimals: {}", balance.decimals);
                }
            }
            Err(e) => {
                println!("{} Account not found: {}", "✗".bright_red(), e);
            }
        }

        Ok(())
    }

    async fn list_accounts(&self, wallet: Option<&str>, keypair_path: Option<&str>) -> Result<()> {
        println!("{} Token Accounts", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;
        let client =
            RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());

        let wallet_pubkey = match wallet {
            Some(w) => validate_pubkey(w)?,
            None => {
                let keypair_path = match keypair_path {
                    Some(p) => p.to_string(),
                    None => {
                        let default = dirs::home_dir()
                            .map(|h| h.join(".config/solana/id.json"))
                            .and_then(|p| p.to_str().map(|s| s.to_string()));

                        match default {
                            Some(p) if std::path::Path::new(&p).exists() => p,
                            _ => {
                                println!(
                                    "{} Wallet address or keypair required!",
                                    "✗".bright_red()
                                );
                                return Ok(());
                            }
                        }
                    }
                };

                let payer = read_keypair_file(&keypair_path).map_err(|e| {
                    SolPrivacyError::Crypto(format!("Failed to read keypair: {}", e))
                })?;
                payer.pubkey()
            }
        };

        println!("  Wallet: {}", wallet_pubkey);
        println!();

        // Get token accounts
        use solana_client::rpc_request::TokenAccountsFilter;

        match client.get_token_accounts_by_owner(
            &wallet_pubkey,
            TokenAccountsFilter::ProgramId(token_2022_program_id()),
        ) {
            Ok(accounts) => {
                if accounts.is_empty() {
                    println!("  No Token-2022 accounts found");
                    println!();
                    println!("  Create one with:");
                    println!("    solprivacy account create --mint <MINT>");
                } else {
                    println!("  {}:", "Token-2022 Accounts".bright_white());
                    for (i, acc) in accounts.iter().enumerate() {
                        let prefix = if i == accounts.len() - 1 {
                            "└─"
                        } else {
                            "├─"
                        };
                        println!("  {} {}", prefix, acc.pubkey);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to fetch accounts: {}", "✗".bright_red(), e);
            }
        }

        Ok(())
    }
}
