use crate::config::AppConfig;
use crate::error::{Result, SolPrivacyError};
use crate::validation::validate_pubkey;
use clap::{Args, Subcommand};
use colored::Colorize;
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::signature::{read_keypair_file, Signer};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::id as token_2022_program_id;
use std::process::Command;

/// Confidential transfer operations for Token-2022
#[derive(Args)]
pub struct ConfidentialCommand {
    #[command(subcommand)]
    pub action: ConfidentialAction,
}

#[derive(Subcommand)]
pub enum ConfidentialAction {
    /// Configure a token account for confidential transfers
    Configure {
        /// Token account address (ATA)
        #[arg(short, long)]
        account: String,

        /// Mint address
        #[arg(short, long)]
        mint: String,

        /// Path to ElGamal keypair JSON for encryption
        #[arg(short, long)]
        elgamal_keypair: Option<String>,

        /// Path to payer keypair
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,

        /// Dry run - simulate without sending
        #[arg(long)]
        dry_run: bool,
    },

    /// Deposit tokens from public balance to pending confidential balance
    Deposit {
        /// Token account address (ATA)
        #[arg(short, long)]
        account: String,

        /// Amount to deposit (in token units)
        #[arg(short = 'n', long)]
        amount: f64,

        /// Mint address
        #[arg(short, long)]
        mint: String,

        /// Decimals (if not auto-detected)
        #[arg(short, long)]
        decimals: Option<u8>,

        /// Path to payer keypair
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,

        /// Dry run - simulate without sending
        #[arg(long)]
        dry_run: bool,
    },

    /// Apply pending balance to available confidential balance
    Apply {
        /// Token account address (ATA)
        #[arg(short, long)]
        account: String,

        /// Path to ElGamal keypair for decryption
        #[arg(short, long)]
        elgamal_keypair: Option<String>,

        /// Path to payer keypair
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,

        /// Dry run - simulate without sending
        #[arg(long)]
        dry_run: bool,
    },

    /// Transfer tokens confidentially (encrypted amounts)
    Transfer {
        /// Mint address
        #[arg(short, long)]
        mint: String,

        /// Recipient address (wallet or ATA)
        #[arg(short, long)]
        to: String,

        /// Amount to transfer (in token units)
        #[arg(short = 'n', long)]
        amount: f64,

        /// Path to sender's ElGamal keypair
        #[arg(short, long)]
        elgamal_keypair: Option<String>,

        /// Path to payer keypair
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,

        /// Dry run - simulate without sending
        #[arg(long)]
        dry_run: bool,
    },

    /// Withdraw tokens from confidential balance to public balance
    Withdraw {
        /// Token account address (ATA)
        #[arg(short, long)]
        account: String,

        /// Amount to withdraw (in token units)
        #[arg(short = 'n', long)]
        amount: f64,

        /// Mint address
        #[arg(short, long)]
        mint: String,

        /// Path to ElGamal keypair for proof generation
        #[arg(short, long)]
        elgamal_keypair: Option<String>,

        /// Path to payer keypair
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,

        /// Dry run - simulate without sending
        #[arg(long)]
        dry_run: bool,
    },

    /// Show confidential balance information
    Balance {
        /// Token account address (ATA) or wallet address
        #[arg(short, long)]
        account: String,

        /// Mint address (required if using wallet address)
        #[arg(short, long)]
        mint: Option<String>,

        /// Path to ElGamal keypair for decryption
        #[arg(short, long)]
        elgamal_keypair: Option<String>,

        /// Show raw encrypted values
        #[arg(long)]
        raw: bool,
    },

    /// Show information about confidential transfer mint configuration
    Info {
        /// Mint address to check
        #[arg(short, long)]
        mint: String,
    },

    /// Show the complete confidential transfer workflow
    Workflow,
}

impl ConfidentialCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            ConfidentialAction::Configure {
                account,
                mint,
                elgamal_keypair,
                keypair,
                dry_run,
            } => {
                self.configure_account(
                    account,
                    mint,
                    elgamal_keypair.as_deref(),
                    keypair.as_deref(),
                    *dry_run,
                )
                .await
            }
            ConfidentialAction::Deposit {
                account,
                amount,
                mint,
                decimals,
                keypair,
                dry_run,
            } => {
                self.deposit(
                    account,
                    *amount,
                    mint,
                    *decimals,
                    keypair.as_deref(),
                    *dry_run,
                )
                .await
            }
            ConfidentialAction::Apply {
                account,
                elgamal_keypair,
                keypair,
                dry_run,
            } => {
                self.apply_pending(
                    account,
                    elgamal_keypair.as_deref(),
                    keypair.as_deref(),
                    *dry_run,
                )
                .await
            }
            ConfidentialAction::Transfer {
                mint,
                to,
                amount,
                elgamal_keypair,
                keypair,
                dry_run,
            } => {
                self.transfer(
                    mint,
                    to,
                    *amount,
                    elgamal_keypair.as_deref(),
                    keypair.as_deref(),
                    *dry_run,
                )
                .await
            }
            ConfidentialAction::Withdraw {
                account,
                amount,
                mint,
                elgamal_keypair,
                keypair,
                dry_run,
            } => {
                self.withdraw(
                    account,
                    *amount,
                    mint,
                    elgamal_keypair.as_deref(),
                    keypair.as_deref(),
                    *dry_run,
                )
                .await
            }
            ConfidentialAction::Balance {
                account,
                mint,
                elgamal_keypair,
                raw,
            } => {
                self.show_balance(account, mint.as_deref(), elgamal_keypair.as_deref(), *raw)
                    .await
            }
            ConfidentialAction::Info { mint } => self.show_info(mint).await,
            ConfidentialAction::Workflow => self.show_workflow().await,
        }
    }

    async fn configure_account(
        &self,
        account: &str,
        mint: &str,
        elgamal_keypair: Option<&str>,
        keypair_path: Option<&str>,
        dry_run: bool,
    ) -> Result<()> {
        println!(
            "{} Configure Confidential Transfer Account",
            "→".bright_cyan()
        );
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;

        let _account_pubkey = validate_pubkey(account)?;
        let _mint_pubkey = validate_pubkey(mint)?;

        println!("  {}:", "Configuration".bright_white());
        println!("  ├─ Account: {}...", &account[..16.min(account.len())]);
        println!("  ├─ Mint: {}...", &mint[..16.min(mint.len())]);
        println!("  └─ Network: {}", config.network);
        println!();

        // Load ElGamal keypair if provided
        if let Some(path) = elgamal_keypair {
            match load_elgamal_pubkey(path) {
                Ok(pk) => {
                    println!(
                        "  {} ElGamal key loaded: {}...",
                        "✓".bright_green(),
                        &pk[..16.min(pk.len())]
                    );
                }
                Err(e) => {
                    println!(
                        "  {} Failed to load ElGamal key: {}",
                        "⚠".bright_yellow(),
                        e
                    );
                }
            }
        } else {
            println!("  {} No ElGamal keypair provided", "ℹ".bright_blue());
            println!("    Generate one with: solprivacy keygen elgamal -o elgamal.json");
        }
        println!();

        // Validate keypair exists
        if !dry_run {
            let _keypair_path = resolve_keypair_path(keypair_path)?;
        }

        // Show what configuration requires
        println!("  {}:", "Configuration Requirements".bright_white());
        println!("  │");
        println!("  │  Configuring an account for confidential transfers requires:");
        println!("  │");
        println!("  │  1. {} ElGamal Keypair", "→".bright_cyan());
        println!("  │     • Generate with: solprivacy keygen elgamal -o elgamal.json");
        println!("  │     • Public key is stored on-chain for encryption");
        println!("  │     • Secret key is needed to decrypt your balance");
        println!("  │");
        println!("  │  2. {} PubkeyValidity ZK Proof", "→".bright_cyan());
        println!("  │     • Proves the ElGamal pubkey is well-formed");
        println!("  │     • Generated using solana-zk-sdk");
        println!("  │");
        println!("  │  3. {} Decryptable Zero Balance", "→".bright_cyan());
        println!("  │     • AES-encrypted zero value for balance initialization");
        println!("  │     • Uses AES-GCM-SIV with key derived from ElGamal secret");
        println!("  │");
        println!("  └─ See: https://solana.com/docs/tokens/extensions/confidential-transfer");
        println!();

        if dry_run {
            println!(
                "{} Dry run - showing configuration steps",
                "ℹ".bright_blue()
            );
        } else {
            println!("{} Executing configuration...", "→".bright_cyan());

            let mut args = vec![
                "configure-confidential-transfer-account",
                "--address",
                account,
            ];
            if let Some(path) = elgamal_keypair {
                args.push("--elgamal-keypair");
                args.push(path);
            }

            self.run_spl_token_command(&args)?;
            println!("{} Configuration complete!", "✓".bright_green());
        }

        Ok(())
    }

    async fn deposit(
        &self,
        account: &str,
        amount: f64,
        mint: &str,
        decimals: Option<u8>,
        keypair_path: Option<&str>,
        dry_run: bool,
    ) -> Result<()> {
        println!("{} Deposit to Confidential Balance", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;

        let _account_pubkey = validate_pubkey(account)?;
        let mint_pubkey = validate_pubkey(mint)?;

        println!("  {}:", "Deposit Details".bright_white());
        println!("  ├─ Account: {}...", &account[..16.min(account.len())]);
        println!("  ├─ Amount: {}", format!("{}", amount).bright_cyan());
        println!("  ├─ Mint: {}...", &mint[..16.min(mint.len())]);
        println!("  └─ Network: {}", config.network);
        println!();

        // Get decimals from mint if not provided (skip RPC call in dry_run if decimals provided)
        let decimals = match decimals {
            Some(d) => d,
            None if dry_run => {
                println!(
                    "  {} Using default decimals: 9 (provide --decimals for accuracy)",
                    "ℹ".bright_blue()
                );
                9
            }
            None => {
                let client = RpcClient::new_with_commitment(
                    config.get_rpc_url(),
                    CommitmentConfig::confirmed(),
                );
                let mint_account = client
                    .get_account(&mint_pubkey)
                    .map_err(|e| SolPrivacyError::Other(format!("Failed to get mint: {}", e)))?;
                if mint_account.data.len() >= 45 {
                    mint_account.data[44]
                } else {
                    9
                }
            }
        };

        let raw_amount = (amount * 10f64.powi(decimals as i32)) as u64;
        println!("  Raw amount: {} (decimals: {})", raw_amount, decimals);
        println!();

        // Validate keypair exists
        if !dry_run {
            let _keypair_path = resolve_keypair_path(keypair_path)?;
        }

        println!("  {}:", "What Deposit Does".bright_white());
        println!("  │");
        println!("  │  1. {} tokens moved from public balance", amount);
        println!("  │  2. Amount is encrypted using account's ElGamal pubkey");
        println!("  │  3. Encrypted amount added to pending balance");
        println!("  │  4. Pending balance must be applied before transfer");
        println!("  │");
        println!("  └─ Deposit is the simplest CT operation (no ZK proofs needed)");
        println!();

        if dry_run {
            println!("{} Dry run - no transaction sent", "ℹ".bright_blue());
            println!();
            println!("  {}:", "Next Step After Deposit".bright_white());
            println!("    Apply pending balance:");
            println!("    solprivacy confidential apply --account {}", account);
        } else {
            println!("{} Executing deposit...", "→".bright_cyan());

            // 1. Load Payer
            let keypair_path_str = resolve_keypair_path(keypair_path)?;
            let payer = read_keypair_file(&keypair_path_str)
                .map_err(|e| SolPrivacyError::Crypto(format!("Failed to read keypair: {}", e)))?;

            // 2. Connect to RPC
            let client =
                RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());

            // 3. Build Instruction
            // spl_token_2022::extension::confidential_transfer::instruction::deposit(
            //   token_program_id,
            //   token_account,
            //   mint,
            //   authority,
            //   amount,
            //   decimals
            // )
            let account_pubkey = validate_pubkey(account)?;
            let mint_pubkey = validate_pubkey(mint)?;

            let deposit_ix =
                spl_token_2022::extension::confidential_transfer::instruction::deposit(
                    &token_2022_program_id(),
                    &account_pubkey,
                    &mint_pubkey,
                    raw_amount,
                    decimals,
                    &payer.pubkey(),
                    &[], // multisig signers
                )
                .map_err(|e| {
                    SolPrivacyError::Other(format!("Failed to create deposit instruction: {}", e))
                })?;

            // 4. Send Transaction
            let blockhash = client
                .get_latest_blockhash()
                .map_err(|e| SolPrivacyError::Rpc(format!("Failed to get blockhash: {}", e)))?;

            let transaction = solana_sdk::transaction::Transaction::new_signed_with_payer(
                &[deposit_ix],
                Some(&payer.pubkey()),
                &[&payer],
                blockhash,
            );

            let signature = client
                .send_and_confirm_transaction(&transaction)
                .map_err(|e| SolPrivacyError::Rpc(format!("Transaction failed: {}", e)))?;

            println!("{} Deposit complete!", "✓".bright_green());
            println!("  Signature: {}", signature);
        }

        Ok(())
    }

    async fn apply_pending(
        &self,
        account: &str,
        elgamal_keypair: Option<&str>,
        keypair_path: Option<&str>,
        dry_run: bool,
    ) -> Result<()> {
        println!("{} Apply Pending Confidential Balance", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;

        let _account_pubkey = validate_pubkey(account)?;

        println!("  {}:", "Apply Details".bright_white());
        println!("  ├─ Account: {}...", &account[..16.min(account.len())]);
        println!("  └─ Network: {}", config.network);
        println!();

        if let Some(path) = elgamal_keypair {
            match load_elgamal_pubkey(path) {
                Ok(pk) => {
                    println!(
                        "  {} ElGamal key: {}...",
                        "✓".bright_green(),
                        &pk[..16.min(pk.len())]
                    );
                }
                Err(e) => {
                    println!("  {} ElGamal key error: {}", "⚠".bright_yellow(), e);
                }
            }
            println!();
        }

        // Validate keypair exists
        if !dry_run {
            let _keypair_path = resolve_keypair_path(keypair_path)?;
        }

        println!("  {}:", "What Apply Does".bright_white());
        println!("  │");
        println!("  │  1. Decrypts pending balance ciphertext (needs ElGamal secret)");
        println!("  │  2. Adds decrypted amount to available confidential balance");
        println!("  │  3. Updates decryptable_available_balance (AES-encrypted)");
        println!("  │  4. Resets pending balance to zero");
        println!("  │");
        println!("  └─ After apply, tokens are available for confidential transfers");
        println!();

        if dry_run {
            println!("{} Dry run - no transaction sent", "ℹ".bright_blue());
        } else {
            println!("{} Executing apply...", "→".bright_cyan());

            let mut args = vec!["apply-pending-balance", "--address", account];
            if let Some(path) = elgamal_keypair {
                args.push("--elgamal-keypair");
                args.push(path);
            }

            self.run_spl_token_command(&args)?;
            println!("{} Pending balance applied!", "✓".bright_green());
        }

        Ok(())
    }

    async fn transfer(
        &self,
        mint: &str,
        to: &str,
        amount: f64,
        elgamal_keypair: Option<&str>,
        keypair_path: Option<&str>,
        dry_run: bool,
    ) -> Result<()> {
        println!("{} Confidential Transfer", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;

        let _mint_pubkey = validate_pubkey(mint)?;
        let _recipient = validate_pubkey(to)?;

        println!("  {}:", "Transfer Details".bright_white());
        println!("  ├─ Mint: {}...", &mint[..16.min(mint.len())]);
        println!("  ├─ To: {}...", &to[..16.min(to.len())]);
        println!(
            "  ├─ Amount: {} (will be encrypted)",
            format!("{}", amount).bright_cyan()
        );
        println!(
            "  ├─ Mode: {}",
            "Confidential (ZK proofs required)".bright_green()
        );
        println!("  └─ Network: {}", config.network);
        println!();

        if let Some(path) = elgamal_keypair {
            match load_elgamal_pubkey(path) {
                Ok(pk) => {
                    println!(
                        "  {} Sender ElGamal key: {}...",
                        "✓".bright_green(),
                        &pk[..16.min(pk.len())]
                    );
                }
                Err(e) => {
                    println!("  {} ElGamal key error: {}", "⚠".bright_yellow(), e);
                }
            }
            println!();
        }

        // Validate keypair exists
        if !dry_run {
            let _keypair_path = resolve_keypair_path(keypair_path)?;
        }

        self.show_transfer_proofs();
        println!();

        if dry_run {
            println!("{} Dry run - no transaction sent", "ℹ".bright_blue());
        } else {
            println!("{} Executing confidential transfer...", "→".bright_cyan());

            let amount_str = amount.to_string();
            let mut args = vec!["transfer", mint, &amount_str, to, "--confidential"];

            if let Some(path) = elgamal_keypair {
                args.push("--elgamal-keypair");
                args.push(path);
            }

            self.run_spl_token_command(&args)?;
            println!("{} Transfer complete!", "✓".bright_green());
        }

        Ok(())
    }

    fn show_transfer_proofs(&self) {
        println!("  {}:", "ZK Proofs Required for Transfer".bright_white());
        println!("  │");
        println!("  │  1. {} Equality Proof", "→".bright_cyan());
        println!("  │     • Proves ciphertext encrypts same value under two keys");
        println!("  │     • Required for sender and recipient encryption");
        println!("  │");
        println!("  │  2. {} Range Proof (Bulletproof)", "→".bright_cyan());
        println!("  │     • Proves encrypted amount is in valid range [0, 2^64)");
        println!("  │     • Prevents negative amounts or overflow");
        println!("  │");
        println!("  │  3. {} Ciphertext Validity Proof", "→".bright_cyan());
        println!("  │     • Proves ciphertext is well-formed ElGamal encryption");
        println!("  │     • Ensures correct cryptographic structure");
        println!("  │");
        println!(
            "  │  4. {} Zero-Balance Proof (if needed)",
            "→".bright_cyan()
        );
        println!("  │     • Proves remaining balance is non-negative");
        println!("  │     • Sender balance >= transfer amount");
        println!("  │");
        println!("  └─ All proofs generated client-side, verified on-chain");
    }

    async fn withdraw(
        &self,
        account: &str,
        amount: f64,
        mint: &str,
        elgamal_keypair: Option<&str>,
        keypair_path: Option<&str>,
        dry_run: bool,
    ) -> Result<()> {
        println!("{} Withdraw from Confidential Balance", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;

        let _account_pubkey = validate_pubkey(account)?;
        let _mint_pubkey = validate_pubkey(mint)?;

        println!("  {}:", "Withdraw Details".bright_white());
        println!("  ├─ Account: {}...", &account[..16.min(account.len())]);
        println!(
            "  ├─ Amount: {} (to public balance)",
            format!("{}", amount).bright_cyan()
        );
        println!("  ├─ Mint: {}...", &mint[..16.min(mint.len())]);
        println!("  └─ Network: {}", config.network);
        println!();

        if let Some(path) = elgamal_keypair {
            match load_elgamal_pubkey(path) {
                Ok(pk) => {
                    println!(
                        "  {} ElGamal key: {}...",
                        "✓".bright_green(),
                        &pk[..16.min(pk.len())]
                    );
                }
                Err(e) => {
                    println!("  {} ElGamal key error: {}", "⚠".bright_yellow(), e);
                }
            }
            println!();
        }

        // Validate keypair exists
        if !dry_run {
            let _keypair_path = resolve_keypair_path(keypair_path)?;
        }

        println!("  {}:", "What Withdraw Does".bright_white());
        println!("  │");
        println!("  │  1. Generate ZK proof that confidential balance >= amount");
        println!("  │  2. Decrypt and subtract amount from confidential balance");
        println!("  │  3. Add amount to public (visible) balance");
        println!("  │  4. Update decryptable_available_balance");
        println!("  │");
        println!("  └─ Requires Range Proof to verify sufficient balance");
        println!();

        if dry_run {
            println!("{} Dry run - no transaction sent", "ℹ".bright_blue());
        } else {
            println!("{} Executing withdraw...", "→".bright_cyan());

            let amount_str = amount.to_string();
            let mut args = vec![
                "withdraw-confidential-tokens",
                mint,
                &amount_str,
                "--address",
                account,
            ];

            if let Some(path) = elgamal_keypair {
                args.push("--elgamal-keypair");
                args.push(path);
            }

            self.run_spl_token_command(&args)?;
            println!("{} Withdraw complete!", "✓".bright_green());
        }

        Ok(())
    }

    async fn show_balance(
        &self,
        account: &str,
        mint: Option<&str>,
        elgamal_keypair: Option<&str>,
        raw: bool,
    ) -> Result<()> {
        println!("{} Confidential Balance", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;
        let client =
            RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());

        let account_pubkey = validate_pubkey(account)?;

        // If mint is provided, calculate ATA
        let token_account = if let Some(mint_str) = mint {
            let mint_pubkey = validate_pubkey(mint_str)?;
            get_associated_token_address_with_program_id(
                &account_pubkey,
                &mint_pubkey,
                &token_2022_program_id(),
            )
        } else {
            account_pubkey
        };

        println!("  Account: {}", token_account);
        println!();

        // Get account data
        match client.get_account(&token_account) {
            Ok(account_data) => {
                let is_token_2022 = account_data.owner == token_2022_program_id();

                if !is_token_2022 {
                    println!("{} Not a Token-2022 account", "✗".bright_red());
                    return Ok(());
                }

                // Get public balance
                if let Ok(balance) = client.get_token_account_balance(&token_account) {
                    println!("  {}:", "Public Balance".bright_white());
                    println!("  ├─ Amount: {}", balance.ui_amount_string.bright_cyan());
                    println!("  └─ Decimals: {}", balance.decimals);
                    println!();
                }

                // Parse confidential transfer extension data
                println!("  {}:", "Confidential Balance".bright_white());

                // Check if account has confidential transfer extension
                // Token account base is 165 bytes, CT extension adds more
                if account_data.data.len() > 165 {
                    println!("  ├─ Status: {}", "CT Extension Present".bright_green());

                    if raw {
                        // Show raw extension data info
                        println!("  ├─ Account data size: {} bytes", account_data.data.len());
                        println!("  ├─ Extension data starts at: byte 165");
                        let extension_size = account_data.data.len() - 165;
                        println!("  └─ Extension size: {} bytes", extension_size);
                    } else {
                        println!(
                            "  ├─ Pending balance: {}",
                            "[encrypted ciphertext]".bright_yellow()
                        );
                        println!(
                            "  ├─ Available balance: {}",
                            "[encrypted ciphertext]".bright_yellow()
                        );
                        println!("  │");

                        if elgamal_keypair.is_some() {
                            println!(
                                "  └─ {} ElGamal key provided - decryption requires ZK SDK",
                                "ℹ".bright_blue()
                            );
                        } else {
                            println!(
                                "  └─ {} Provide --elgamal-keypair to enable decryption",
                                "ℹ".bright_blue()
                            );
                        }
                    }
                } else {
                    println!(
                        "  └─ Status: {} (account not configured)",
                        "No CT Extension".bright_yellow()
                    );
                    println!();
                    println!("  Configure with:");
                    println!(
                        "    solprivacy confidential configure --account {} --mint <MINT>",
                        account
                    );
                }
            }
            Err(e) => {
                println!("{} Failed to get account: {}", "✗".bright_red(), e);
            }
        }

        Ok(())
    }

    async fn show_info(&self, mint: &str) -> Result<()> {
        println!("{} Confidential Transfer Mint Info", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();

        let config = AppConfig::load()?;
        let client =
            RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());

        let mint_pubkey = validate_pubkey(mint)?;

        println!("  Mint: {}", mint_pubkey);
        println!();

        match client.get_account(&mint_pubkey) {
            Ok(account_data) => {
                let is_token_2022 = account_data.owner == token_2022_program_id();

                println!("  {}:", "Mint Info".bright_white());
                println!(
                    "  ├─ Program: {}",
                    if is_token_2022 {
                        "Token-2022 ✓".bright_green()
                    } else {
                        "Unknown".bright_yellow()
                    }
                );
                println!("  ├─ Data size: {} bytes", account_data.data.len());

                // Standard mint is 82 bytes, with extensions it's larger
                if account_data.data.len() > 82 {
                    println!("  ├─ Extensions: {}", "Present".bright_green());

                    // Estimate if CT extension is present based on size
                    // ConfidentialTransferMint extension adds significant data
                    if account_data.data.len() >= 200 {
                        println!("  │  └─ Likely has ConfidentialTransferMint extension");
                    }
                } else {
                    println!("  ├─ Extensions: {}", "None".bright_yellow());
                }

                // Get decimals
                if account_data.data.len() >= 45 {
                    let decimals = account_data.data[44];
                    println!("  └─ Decimals: {}", decimals);
                }

                println!();

                if is_token_2022 && account_data.data.len() > 82 {
                    println!(
                        "  {} This mint likely supports confidential transfers",
                        "✓".bright_green()
                    );
                    println!();
                    println!("  {}:", "Verify with Solana CLI".bright_white());
                    println!("    spl-token display {}", mint);
                } else {
                    println!("  {} Mint may not have CT extension", "⚠".bright_yellow());
                    println!();
                    println!("  Create a new mint with CT:");
                    println!("    solprivacy mint create --name MyToken --symbol MTK --decimals 9");
                }
            }
            Err(e) => {
                println!("{} Failed to get mint: {}", "✗".bright_red(), e);
            }
        }

        Ok(())
    }

    fn run_spl_token_command(&self, args: &[&str]) -> Result<()> {
        println!("Running: spl-token {}", args.join(" "));
        let output = Command::new("spl-token")
            .args(args)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .map_err(|e| {
                SolPrivacyError::ToolMissing(format!(
                    "Failed to execute spl-token: {}. Is it installed?",
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(SolPrivacyError::CommandFailed(
                "spl-token command failed".to_string(),
            ));
        }
        Ok(())
    }

    async fn show_workflow(&self) -> Result<()> {
        println!(
            "{} Token-2022 Confidential Transfer Workflow",
            "→".bright_cyan()
        );
        println!("{}", "─".repeat(60).bright_black());
        println!();

        println!("  {}:", "Complete Privacy Workflow".bright_white());
        println!();

        println!("  {} SETUP PHASE", "1.".bright_cyan());
        println!("  │");
        println!("  │  Generate ElGamal keypair for encryption:");
        println!("  │  └─ solprivacy keygen elgamal -o keys/elgamal.json");
        println!("  │");
        println!("  │  Generate auditor keypair (optional, for compliance):");
        println!("  │  └─ solprivacy keygen auditor -o keys/auditor.json");
        println!("  │");
        println!();

        println!("  {} CREATE CONFIDENTIAL TOKEN", "2.".bright_cyan());
        println!("  │");
        println!("  │  Create mint with ConfidentialTransferMint extension:");
        println!("  │  └─ solprivacy mint create --name MyToken --symbol MTK \\");
        println!("  │       --supply 1000000 --auditor keys/auditor.json");
        println!("  │");
        println!();

        println!("  {} CONFIGURE ACCOUNT", "3.".bright_cyan());
        println!("  │");
        println!("  │  Enable confidential transfers on token account:");
        println!("  │  └─ solprivacy confidential configure \\");
        println!("  │       --account <ATA> --mint <MINT> \\");
        println!("  │       --elgamal-keypair keys/elgamal.json");
        println!("  │");
        println!();

        println!("  {} DEPOSIT TO CONFIDENTIAL BALANCE", "4.".bright_cyan());
        println!("  │");
        println!("  │  Move tokens from public to encrypted balance:");
        println!("  │  └─ solprivacy confidential deposit \\");
        println!("  │       --account <ATA> --mint <MINT> --amount 1000");
        println!("  │");
        println!();

        println!("  {} APPLY PENDING BALANCE", "5.".bright_cyan());
        println!("  │");
        println!("  │  Make deposited tokens available for transfer:");
        println!("  │  └─ solprivacy confidential apply \\");
        println!("  │       --account <ATA> --elgamal-keypair keys/elgamal.json");
        println!("  │");
        println!();

        println!("  {} CONFIDENTIAL TRANSFER", "6.".bright_cyan());
        println!("  │");
        println!("  │  Transfer with encrypted amounts (ZK proofs):");
        println!("  │  └─ solprivacy confidential transfer \\");
        println!("  │       --mint <MINT> --to <RECIPIENT> --amount 100 \\");
        println!("  │       --elgamal-keypair keys/elgamal.json");
        println!("  │");
        println!();

        println!("  {} WITHDRAW TO PUBLIC (Optional)", "7.".bright_cyan());
        println!("  │");
        println!("  │  Convert confidential balance back to public:");
        println!("  │  └─ solprivacy confidential withdraw \\");
        println!("  │       --account <ATA> --mint <MINT> --amount 50 \\");
        println!("  │       --elgamal-keypair keys/elgamal.json");
        println!();

        println!("  {}:", "Key Concepts".bright_white());
        println!("  │");
        println!(
            "  ├─ {} Public balance is visible on-chain",
            "•".bright_cyan()
        );
        println!(
            "  ├─ {} Confidential balance is ElGamal encrypted",
            "•".bright_cyan()
        );
        println!(
            "  ├─ {} Pending balance: recently received (needs apply)",
            "•".bright_cyan()
        );
        println!(
            "  ├─ {} Available balance: ready for transfer/withdraw",
            "•".bright_cyan()
        );
        println!(
            "  └─ {} All confidential ops require ZK proofs",
            "•".bright_cyan()
        );
        println!();

        println!("  {}:", "Documentation".bright_white());
        println!("    https://solana.com/docs/tokens/extensions/confidential-transfer");

        Ok(())
    }
}

/// Load ElGamal public key from keypair JSON file
fn load_elgamal_pubkey(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| SolPrivacyError::Crypto(format!("Failed to read file: {}", e)))?;

    let keypair: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| SolPrivacyError::Crypto(format!("Failed to parse JSON: {}", e)))?;

    keypair
        .get("public_key")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| SolPrivacyError::Crypto("No public_key field found".to_string()))
}

/// Resolve keypair path from option or default location
fn resolve_keypair_path(keypair_path: Option<&str>) -> Result<String> {
    match keypair_path {
        Some(p) => Ok(p.to_string()),
        None => {
            let default = dirs::home_dir()
                .map(|h| h.join(".config/solana/id.json"))
                .and_then(|p| p.to_str().map(|s| s.to_string()));

            match default {
                Some(p) if std::path::Path::new(&p).exists() => Ok(p),
                _ => Err(SolPrivacyError::Config(
                    "Keypair required. Use --keypair <path> or set SOLANA_KEYPAIR env".to_string(),
                )),
            }
        }
    }
}
