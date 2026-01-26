use crate::config::AppConfig;
use crate::error::{Result, SolPrivacyError};
use clap::{Args, Subcommand};
use colored::Colorize;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::{
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{read_keypair_file, write_keypair_file, Keypair, Signer},
};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// Wallet management commands
#[derive(Args)]
pub struct WalletCommand {
    #[command(subcommand)]
    pub action: WalletAction,
}

#[derive(Subcommand)]
pub enum WalletAction {
    /// Generate a new wallet keypair
    New {
        /// Name for the wallet (stored in ~/.config/solprivacy/wallets/)
        #[arg(short, long, default_value = "default")]
        name: String,

        /// Force overwrite if wallet exists
        #[arg(short, long)]
        force: bool,
    },

    /// Import a wallet from private key or mnemonic
    Import {
        /// Wallet name
        #[arg(short, long, default_value = "imported")]
        name: String,

        /// Private key in base58 format
        #[arg(long)]
        private_key: Option<String>,

        /// Path to existing keypair JSON file
        #[arg(long)]
        keypair: Option<String>,
    },

    /// Show wallet balance (SOL and tokens)
    Balance {
        /// Wallet name or address (defaults to 'default' wallet)
        #[arg()]
        wallet: Option<String>,
    },

    /// Request SOL airdrop (devnet/testnet only)
    Airdrop {
        /// Amount of SOL to request (default: 2)
        #[arg(short, long, default_value = "2")]
        amount: f64,

        /// Wallet name or address
        #[arg()]
        wallet: Option<String>,
    },

    /// List all configured wallets
    List,

    /// Show wallet public key
    Address {
        /// Wallet name (defaults to 'default')
        #[arg()]
        name: Option<String>,
    },
}

impl WalletCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            WalletAction::New { name, force } => self.create_wallet(name, *force),
            WalletAction::Import {
                name,
                private_key,
                keypair,
            } => self.import_wallet(name, private_key.as_deref(), keypair.as_deref()),
            WalletAction::Balance { wallet } => self.show_balance(wallet.as_deref()).await,
            WalletAction::Airdrop { amount, wallet } => {
                self.request_airdrop(*amount, wallet.as_deref()).await
            }
            WalletAction::List => self.list_wallets(),
            WalletAction::Address { name } => self.show_address(name.as_deref()),
        }
    }

    fn wallets_dir() -> PathBuf {
        AppConfig::config_dir().join("wallets")
    }

    fn wallet_path(name: &str) -> PathBuf {
        Self::wallets_dir().join(format!("{}.json", name))
    }

    fn create_wallet(&self, name: &str, force: bool) -> Result<()> {
        println!("{} Creating new wallet...", "→".bright_cyan());
        println!();

        let wallet_path = Self::wallet_path(name);

        if wallet_path.exists() && !force {
            println!("{} Wallet '{}' already exists!", "✗".bright_red(), name);
            println!("  Use --force to overwrite");
            return Ok(());
        }

        // Create wallets directory
        fs::create_dir_all(Self::wallets_dir())?;

        // Generate new keypair
        let keypair = Keypair::new();

        // Save keypair
        write_keypair_file(&keypair, &wallet_path)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to save wallet: {}", e)))?;

        // Set restrictive file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&wallet_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = fs::set_permissions(&wallet_path, perms);
            }
        }

        println!("{} Wallet created!", "✓".bright_green());
        println!();
        println!("  {}:", "Wallet Details".bright_white());
        println!("  ├─ Name: {}", name.bright_cyan());
        println!(
            "  ├─ Address: {}",
            keypair.pubkey().to_string().bright_yellow()
        );
        println!("  └─ Path: {}", wallet_path.display());
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. Get devnet SOL: solprivacy wallet airdrop");
        println!("    2. Check balance: solprivacy wallet balance");

        Ok(())
    }

    fn import_wallet(
        &self,
        name: &str,
        private_key: Option<&str>,
        keypair_path: Option<&str>,
    ) -> Result<()> {
        println!("{} Importing wallet...", "→".bright_cyan());
        println!();

        let wallet_path = Self::wallet_path(name);
        fs::create_dir_all(Self::wallets_dir())?;

        let keypair = if let Some(pk) = private_key {
            // Import from base58 private key
            let bytes = bs58::decode(pk)
                .into_vec()
                .map_err(|e| SolPrivacyError::Crypto(format!("Invalid base58: {}", e)))?;

            // SDK v3: Use new_from_array for 32-byte secret, or from_base58_string
            if bytes.len() == 32 {
                let secret_array: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| SolPrivacyError::Crypto("Invalid key length".to_string()))?;
                Keypair::new_from_array(secret_array)
            } else if bytes.len() == 64 {
                // Full keypair (secret + public)
                let secret_array: [u8; 32] = bytes[..32]
                    .try_into()
                    .map_err(|_| SolPrivacyError::Crypto("Invalid key length".to_string()))?;
                Keypair::new_from_array(secret_array)
            } else {
                return Err(SolPrivacyError::Crypto(format!(
                    "Invalid key length: {} bytes (expected 32 or 64)",
                    bytes.len()
                )));
            }
        } else if let Some(path) = keypair_path {
            // Import from existing keypair file
            read_keypair_file(path)
                .map_err(|e| SolPrivacyError::Crypto(format!("Failed to read keypair: {}", e)))?
        } else {
            println!("{} Provide --private-key or --keypair!", "✗".bright_red());
            return Ok(());
        };

        // Save keypair
        write_keypair_file(&keypair, &wallet_path)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to save wallet: {}", e)))?;

        // Set restrictive file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&wallet_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = fs::set_permissions(&wallet_path, perms);
            }
        }

        println!("{} Wallet imported!", "✓".bright_green());
        println!();
        println!("  {}:", "Wallet Details".bright_white());
        println!("  ├─ Name: {}", name.bright_cyan());
        println!(
            "  ├─ Address: {}",
            keypair.pubkey().to_string().bright_yellow()
        );
        println!("  └─ Path: {}", wallet_path.display());

        Ok(())
    }

    async fn show_balance(&self, wallet: Option<&str>) -> Result<()> {
        println!("{} Wallet Balance", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;
        let client =
            RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());

        // Resolve wallet address
        let pubkey = self.resolve_wallet_address(wallet)?;

        println!("  Address: {}", pubkey.to_string().bright_cyan());
        println!("  Network: {}", config.network);
        println!();

        // Get SOL balance
        match client.get_balance(&pubkey) {
            Ok(balance) => {
                let sol = balance as f64 / LAMPORTS_PER_SOL as f64;
                println!("  {}:", "SOL Balance".bright_white());
                println!("  └─ {} SOL", format!("{:.9}", sol).bright_green());
            }
            Err(e) => {
                println!("{} Failed to get balance: {}", "✗".bright_red(), e);
            }
        }

        // Get token accounts
        println!();
        println!("  {}:", "Token Accounts".bright_white());

        use solana_client::rpc_request::TokenAccountsFilter;
        use spl_token_2022::id as token_2022_program_id;

        match client.get_token_accounts_by_owner(
            &pubkey,
            TokenAccountsFilter::ProgramId(token_2022_program_id()),
        ) {
            Ok(accounts) => {
                if accounts.is_empty() {
                    println!("  └─ No Token-2022 accounts");
                } else {
                    for (i, acc) in accounts.iter().enumerate() {
                        let prefix = if i == accounts.len() - 1 {
                            "└─"
                        } else {
                            "├─"
                        };
                        if let Ok(balance) = client
                            .get_token_account_balance(&Pubkey::from_str(&acc.pubkey).unwrap())
                        {
                            println!(
                                "  {} {} ({})",
                                prefix,
                                balance.ui_amount_string,
                                &acc.pubkey[..16]
                            );
                        }
                    }
                }
            }
            Err(_) => {
                println!("  └─ No Token-2022 accounts");
            }
        }

        Ok(())
    }

    async fn request_airdrop(&self, amount: f64, wallet: Option<&str>) -> Result<()> {
        println!("{} Requesting Airdrop", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;

        if config.network == "mainnet" {
            println!("{} Airdrop not available on mainnet!", "✗".bright_red());
            return Ok(());
        }

        let client =
            RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());
        let pubkey = self.resolve_wallet_address(wallet)?;

        let lamports = (amount * LAMPORTS_PER_SOL as f64) as u64;

        println!("  Address: {}", pubkey);
        println!("  Amount: {} SOL", amount);
        println!("  Network: {}", config.network);
        println!();

        println!("{} Requesting airdrop...", "→".bright_cyan());

        match client.request_airdrop(&pubkey, lamports) {
            Ok(signature) => {
                println!("{} Airdrop requested!", "✓".bright_green());
                println!();
                println!("  Signature: {}", signature);
                println!();

                // Wait for confirmation
                println!("{} Waiting for confirmation...", "→".bright_cyan());

                match client.confirm_transaction(&signature) {
                    Ok(true) => {
                        println!("{} Airdrop confirmed!", "✓".bright_green());

                        // Show new balance
                        if let Ok(balance) = client.get_balance(&pubkey) {
                            let sol = balance as f64 / LAMPORTS_PER_SOL as f64;
                            println!();
                            println!(
                                "  New Balance: {} SOL",
                                format!("{:.9}", sol).bright_green()
                            );
                        }
                    }
                    _ => {
                        println!(
                            "{} Confirmation pending - check balance later",
                            "⚠".bright_yellow()
                        );
                    }
                }
            }
            Err(e) => {
                println!("{} Airdrop failed: {}", "✗".bright_red(), e);
                println!();
                println!("  Possible reasons:");
                println!("    • Rate limited (wait a few minutes)");
                println!("    • Network congestion");
                println!("    • Invalid address");
            }
        }

        Ok(())
    }

    fn list_wallets(&self) -> Result<()> {
        println!("{} Configured Wallets", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let wallets_dir = Self::wallets_dir();

        if !wallets_dir.exists() {
            println!("  No wallets configured yet.");
            println!();
            println!("  Create one with: solprivacy wallet new");
            return Ok(());
        }

        let entries: Vec<_> = fs::read_dir(&wallets_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
            .collect();

        if entries.is_empty() {
            println!("  No wallets configured yet.");
            println!();
            println!("  Create one with: solprivacy wallet new");
            return Ok(());
        }

        println!("  {}:", "Wallets".bright_white());

        for (i, entry) in entries.iter().enumerate() {
            let path = entry.path();
            let name = path.file_stem().unwrap().to_string_lossy();
            let prefix = if i == entries.len() - 1 {
                "└─"
            } else {
                "├─"
            };

            if let Ok(keypair) = read_keypair_file(&path) {
                println!("  {} {} → {}", prefix, name.bright_cyan(), keypair.pubkey());
            } else {
                println!("  {} {} → (invalid keypair)", prefix, name.bright_red());
            }
        }

        Ok(())
    }

    fn show_address(&self, name: Option<&str>) -> Result<()> {
        let wallet_name = name.unwrap_or("default");
        let wallet_path = Self::wallet_path(wallet_name);

        if !wallet_path.exists() {
            println!("{} Wallet '{}' not found!", "✗".bright_red(), wallet_name);
            println!(
                "  Create one with: solprivacy wallet new --name {}",
                wallet_name
            );
            return Ok(());
        }

        let keypair = read_keypair_file(&wallet_path)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to read wallet: {}", e)))?;

        // Just print the address for scripting use
        println!("{}", keypair.pubkey());

        Ok(())
    }

    fn resolve_wallet_address(&self, wallet: Option<&str>) -> Result<Pubkey> {
        match wallet {
            Some(w) => {
                // Try as pubkey first
                if let Ok(pubkey) = Pubkey::from_str(w) {
                    return Ok(pubkey);
                }

                // Try as wallet name
                let wallet_path = Self::wallet_path(w);
                if wallet_path.exists() {
                    let keypair = read_keypair_file(&wallet_path).map_err(|e| {
                        SolPrivacyError::Crypto(format!("Failed to read wallet: {}", e))
                    })?;
                    return Ok(keypair.pubkey());
                }

                Err(SolPrivacyError::Other(format!("Wallet '{}' not found", w)))
            }
            None => {
                // Use default wallet
                let wallet_path = Self::wallet_path("default");
                if wallet_path.exists() {
                    let keypair = read_keypair_file(&wallet_path).map_err(|e| {
                        SolPrivacyError::Crypto(format!("Failed to read wallet: {}", e))
                    })?;
                    Ok(keypair.pubkey())
                } else {
                    // Try Solana CLI default
                    let solana_default = dirs::home_dir()
                        .map(|h| h.join(".config/solana/id.json"))
                        .filter(|p| p.exists());

                    if let Some(path) = solana_default {
                        let keypair = read_keypair_file(&path).map_err(|e| {
                            SolPrivacyError::Crypto(format!("Failed to read keypair: {}", e))
                        })?;
                        Ok(keypair.pubkey())
                    } else {
                        Err(SolPrivacyError::Other(
                            "No default wallet. Create one with: solprivacy wallet new".to_string(),
                        ))
                    }
                }
            }
        }
    }
}
