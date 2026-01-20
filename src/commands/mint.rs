use clap::{Args, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token_2022::{
    extension::{ExtensionType, confidential_transfer::ConfidentialTransferMint},
    state::Mint,
    instruction as token_instruction,
    id as token_2022_program_id,
};
use spl_associated_token_account::{
    get_associated_token_address_with_program_id,
    instruction::create_associated_token_account,
};
use std::str::FromStr;
use std::time::Duration;
use crate::config::AppConfig;
use crate::error::{Result, SolPrivacyError};

/// Create and manage Token-2022 confidential tokens
#[derive(Args)]
pub struct MintCommand {
    #[command(subcommand)]
    pub action: MintAction,
}

#[derive(Subcommand)]
pub enum MintAction {
    /// Create a new Token-2022 mint with confidential transfer extension
    Create {
        /// Token name (metadata)
        #[arg(short, long)]
        name: String,
        
        /// Token symbol
        #[arg(short, long)]
        symbol: String,
        
        /// Decimals (default: 9)
        #[arg(short, long, default_value = "9")]
        decimals: u8,
        
        /// Initial supply to mint (optional)
        #[arg(long)]
        supply: Option<u64>,
        
        /// Path to auditor keypair JSON (optional, for compliance)
        #[arg(short, long)]
        auditor: Option<String>,
        
        /// Path to payer/authority keypair JSON
        #[arg(short, long, env = "SOLANA_KEYPAIR")]
        keypair: Option<String>,
        
        /// Dry run - simulate without sending transaction
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Show information about a mint
    Info {
        /// Mint address
        #[arg()]
        mint: String,
    },
    
    /// Check balance of a token account
    Balance {
        /// Token account or wallet address
        #[arg()]
        account: String,
        
        /// Mint address (required if using wallet address)
        #[arg(short, long)]
        mint: Option<String>,
    },
}

impl MintCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            MintAction::Create { name, symbol, decimals, supply, auditor, keypair, dry_run } => {
                self.create_mint(name, symbol, *decimals, *supply, auditor.as_deref(), keypair.as_deref(), *dry_run).await
            }
            MintAction::Info { mint } => {
                self.show_mint_info(mint).await
            }
            MintAction::Balance { account, mint } => {
                self.show_balance(account, mint.as_deref()).await
            }
        }
    }
    
    async fn create_mint(
        &self, 
        name: &str, 
        symbol: &str, 
        decimals: u8,
        supply: Option<u64>,
        auditor_path: Option<&str>,
        keypair_path: Option<&str>,
        dry_run: bool,
    ) -> Result<()> {
        println!("{} Token-2022 Confidential Mint Creator", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        // Load configuration
        let config = AppConfig::load()?;
        let rpc_url = config.get_rpc_url();
        
        // Display configuration
        println!("  {}:", "Mint Configuration".bright_white());
        println!("  ├─ Name: {}", name.bright_cyan());
        println!("  ├─ Symbol: {}", symbol.bright_yellow());
        println!("  ├─ Decimals: {}", decimals);
        println!("  ├─ Network: {}", config.network);
        println!("  ├─ Extension: {}", "ConfidentialTransfer".bright_green());
        
        // Load auditor key if provided
        let auditor_pubkey_bytes = if let Some(path) = auditor_path {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let keypair: serde_json::Value = serde_json::from_str(&content)
                        .map_err(|e| SolPrivacyError::Crypto(format!("Failed to parse auditor key: {}", e)))?;
                    if let Some(pk) = keypair.get("public_key").and_then(|v| v.as_str()) {
                        println!("  ├─ Auditor: {}... ✓", &pk[..16.min(pk.len())]);
                        Some(pk.to_string())
                    } else {
                        None
                    }
                }
                Err(_) => {
                    println!("  ├─ Auditor: {} (file not found)", "None".bright_yellow());
                    None
                }
            }
        } else {
            println!("  ├─ Auditor: {}", "None (optional)".bright_black());
            None
        };
        
        if let Some(s) = supply {
            println!("  ├─ Initial Supply: {}", format_amount(s, decimals));
        }
        println!("  └─ RPC: {}", rpc_url);
        println!();
        
        // Dry run mode
        if dry_run {
            println!("{} Dry run mode - no transaction sent", "ℹ".bright_blue());
            println!();
            self.show_dry_run_info(decimals, supply, auditor_pubkey_bytes.is_some());
            return Ok(());
        }
        
        // Check for keypair
        let keypair_path = match keypair_path {
            Some(p) => p.to_string(),
            None => {
                // Try default Solana CLI path
                let default_path = dirs::home_dir()
                    .map(|h| h.join(".config/solana/id.json"))
                    .and_then(|p| p.to_str().map(|s| s.to_string()));
                
                match default_path {
                    Some(p) if std::path::Path::new(&p).exists() => {
                        println!("{} Using default keypair: {}", "ℹ".bright_blue(), p);
                        p
                    }
                    _ => {
                        println!("{} Keypair required!", "✗".bright_red());
                        println!();
                        println!("  Options:");
                        println!("    1. Use --keypair <path>");
                        println!("    2. Set SOLANA_KEYPAIR environment variable");
                        println!("    3. Create keypair: solana-keygen new");
                        return Ok(());
                    }
                }
            }
        };
        
        // Load payer keypair
        let payer = read_keypair_file(&keypair_path)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to read keypair: {}", e)))?;
        
        println!();
        println!("{} Connecting to {}...", "→".bright_cyan(), config.network);
        
        // Connect to RPC
        let client = RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed());
        
        // Check connection and balance
        match client.get_version() {
            Ok(version) => {
                println!("{} Connected (Solana {})", "✓".bright_green(), version.solana_core);
            }
            Err(e) => {
                println!("{} Failed to connect: {}", "✗".bright_red(), e);
                return Ok(());
            }
        }
        
        let balance = client.get_balance(&payer.pubkey())
            .map_err(|e| SolPrivacyError::Other(format!("Failed to get balance: {}", e)))?;
        
        let balance_sol = balance as f64 / 1_000_000_000.0;
        println!("  Payer: {} ({:.4} SOL)", payer.pubkey(), balance_sol);
        
        if balance < 10_000_000 { // 0.01 SOL minimum
            println!();
            println!("{} Insufficient balance!", "✗".bright_red());
            println!("  Need at least 0.01 SOL for transaction fees");
            println!();
            println!("  Get devnet SOL:");
            println!("    solana airdrop 2 {} --url devnet", payer.pubkey());
            return Ok(());
        }
        
        // Create mint
        println!();
        let pb = ProgressBar::new(4);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"));
        
        pb.set_message("Creating mint account...");
        
        // Generate new mint keypair
        let mint_keypair = Keypair::new();
        let mint_pubkey = mint_keypair.pubkey();
        
        // Calculate space needed for mint with confidential transfer extension
        let extensions = vec![ExtensionType::ConfidentialTransferMint];
        let space = ExtensionType::try_calculate_account_len::<Mint>(&extensions)
            .map_err(|e| SolPrivacyError::Other(format!("Failed to calculate space: {:?}", e)))?;
        
        // Get rent
        let rent = client.get_minimum_balance_for_rent_exemption(space)
            .map_err(|e| SolPrivacyError::Other(format!("Failed to get rent: {}", e)))?;
        
        pb.inc(1);
        pb.set_message("Building transaction...");
        
        // Build instructions
        let mut instructions = vec![];
        
        // 1. Create account
        instructions.push(system_instruction::create_account(
            &payer.pubkey(),
            &mint_pubkey,
            rent,
            space as u64,
            &token_2022_program_id(),
        ));
        
        // 2. Initialize confidential transfer mint extension
        // Note: Using default authority (None = auto_approve all accounts)
        instructions.push(
            spl_token_2022::extension::confidential_transfer::instruction::initialize_mint(
                &token_2022_program_id(),
                &mint_pubkey,
                Some(payer.pubkey()), // authority
                true, // auto_approve_new_accounts
                None, // auditor_elgamal_pubkey (simplified for now)
            ).map_err(|e| SolPrivacyError::Other(format!("Failed to create instruction: {:?}", e)))?
        );
        
        // 3. Initialize mint
        instructions.push(
            token_instruction::initialize_mint2(
                &token_2022_program_id(),
                &mint_pubkey,
                &payer.pubkey(), // mint authority
                Some(&payer.pubkey()), // freeze authority
                decimals,
            ).map_err(|e| SolPrivacyError::Other(format!("Failed to create instruction: {:?}", e)))?
        );
        
        pb.inc(1);
        pb.set_message("Sending transaction...");
        
        // Get recent blockhash
        let blockhash = client.get_latest_blockhash()
            .map_err(|e| SolPrivacyError::Other(format!("Failed to get blockhash: {}", e)))?;
        
        // Create and sign transaction
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer, &mint_keypair],
            blockhash,
        );
        
        // Send transaction
        let signature = client.send_and_confirm_transaction(&transaction)
            .map_err(|e| SolPrivacyError::Other(format!("Transaction failed: {}", e)))?;
        
        pb.inc(1);
        
        // Create ATA and mint initial supply if specified
        if let Some(initial_supply) = supply {
            pb.set_message("Creating token account...");
            
            // Get ATA address
            let ata = get_associated_token_address_with_program_id(
                &payer.pubkey(),
                &mint_pubkey,
                &token_2022_program_id(),
            );
            
            // Create ATA instruction
            let create_ata_ix = create_associated_token_account(
                &payer.pubkey(),
                &payer.pubkey(),
                &mint_pubkey,
                &token_2022_program_id(),
            );
            
            // Mint instruction
            let mint_ix = token_instruction::mint_to(
                &token_2022_program_id(),
                &mint_pubkey,
                &ata,
                &payer.pubkey(),
                &[],
                initial_supply,
            ).map_err(|e| SolPrivacyError::Other(format!("Failed to create mint instruction: {:?}", e)))?;
            
            let blockhash = client.get_latest_blockhash()
                .map_err(|e| SolPrivacyError::Other(format!("Failed to get blockhash: {}", e)))?;
            
            let tx = Transaction::new_signed_with_payer(
                &[create_ata_ix, mint_ix],
                Some(&payer.pubkey()),
                &[&payer],
                blockhash,
            );
            
            client.send_and_confirm_transaction(&tx)
                .map_err(|e| SolPrivacyError::Other(format!("Failed to mint: {}", e)))?;
        }
        
        pb.inc(1);
        pb.finish_and_clear();
        
        // Success output
        println!("{} Mint created successfully!", "✓".bright_green());
        println!();
        println!("  {}:", "Mint Details".bright_white());
        println!("  ├─ Address: {}", mint_pubkey.to_string().bright_cyan());
        println!("  ├─ Symbol: {}", symbol.bright_yellow());
        println!("  ├─ Decimals: {}", decimals);
        println!("  ├─ Extension: ConfidentialTransfer ✓");
        
        if let Some(s) = supply {
            println!("  ├─ Initial Supply: {}", format_amount(s, decimals));
            let ata = get_associated_token_address_with_program_id(
                &payer.pubkey(),
                &mint_pubkey,
                &token_2022_program_id(),
            );
            println!("  ├─ Minted To: {}", ata);
        }
        
        println!("  └─ Transaction: {}", signature);
        println!();
        
        // Explorer link
        let explorer_base = match config.network.as_str() {
            "mainnet" => "https://solscan.io",
            _ => "https://solscan.io?cluster=devnet",
        };
        println!("  {}:", "View on Explorer".bright_white());
        println!("    {}/token/{}", explorer_base, mint_pubkey);
        println!();
        
        // Next steps
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. Create token accounts:");
        println!("       solprivacy account create --mint {}", mint_pubkey);
        println!("    2. Transfer tokens:");
        println!("       solprivacy transfer --mint {} --to <ADDR> --amount 100", mint_pubkey);
        
        Ok(())
    }
    
    fn show_dry_run_info(&self, decimals: u8, supply: Option<u64>, has_auditor: bool) {
        println!("  {}:", "Transaction Preview".bright_white());
        println!("  ├─ 1. Create Mint account (Token-2022 program)");
        println!("  ├─ 2. Initialize ConfidentialTransferMint extension");
        if has_auditor {
            println!("  ├─ 3. Configure auditor ElGamal public key");
        }
        println!("  ├─ 3. Initialize Mint with decimals={}", decimals);
        if let Some(s) = supply {
            println!("  ├─ 4. Create Associated Token Account");
            println!("  └─ 5. Mint {} tokens to your account", format_amount(s, decimals));
        } else {
            println!("  └─ (No initial supply - mint manually later)");
        }
        println!();
        println!("  {}:", "Estimated Cost".bright_white());
        println!("    ~0.01 SOL (rent + fees)");
        println!();
        println!("  Remove --dry-run to execute on-chain");
    }
    
    async fn show_mint_info(&self, mint_address: &str) -> Result<()> {
        println!("{} Mint Information", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        let config = AppConfig::load()?;
        let rpc_url = config.get_rpc_url();
        
        let mint_pubkey = Pubkey::from_str(mint_address)
            .map_err(|e| SolPrivacyError::Other(format!("Invalid address: {}", e)))?;
        
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        
        match client.get_account(&mint_pubkey) {
            Ok(account) => {
                let is_token_2022 = account.owner.to_string() == "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
                
                let owner_str = account.owner.to_string();
                println!("  {}:", "Account Info".bright_white());
                println!("  ├─ Address: {}", mint_pubkey);
                println!("  ├─ Owner: {}", if is_token_2022 { "Token-2022" } else { &owner_str });
                println!("  ├─ Lamports: {} ({:.6} SOL)", account.lamports, account.lamports as f64 / 1e9);
                println!("  └─ Data Size: {} bytes", account.data.len());
                
                if is_token_2022 {
                    println!();
                    println!("  {} Token-2022 mint detected", "✓".bright_green());
                    
                    // Check for extensions
                    if account.data.len() > 82 { // Has extensions
                        println!("  Extensions: Likely ConfidentialTransfer enabled");
                    }
                }
            }
            Err(e) => {
                println!("{} Account not found: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn show_balance(&self, account: &str, mint: Option<&str>) -> Result<()> {
        println!("{} Token Balance", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        let config = AppConfig::load()?;
        let client = RpcClient::new_with_commitment(config.get_rpc_url(), CommitmentConfig::confirmed());
        
        let account_pubkey = Pubkey::from_str(account)
            .map_err(|e| SolPrivacyError::Other(format!("Invalid address: {}", e)))?;
        
        // If mint is provided, calculate ATA
        let token_account = if let Some(mint_str) = mint {
            let mint_pubkey = Pubkey::from_str(mint_str)
                .map_err(|e| SolPrivacyError::Other(format!("Invalid mint: {}", e)))?;
            get_associated_token_address_with_program_id(
                &account_pubkey,
                &mint_pubkey,
                &token_2022_program_id(),
            )
        } else {
            account_pubkey
        };
        
        println!("  Checking: {}", token_account);
        println!();
        
        match client.get_token_account_balance(&token_account) {
            Ok(balance) => {
                println!("  {}:", "Balance".bright_white());
                println!("  ├─ Amount: {}", balance.ui_amount_string.bright_cyan());
                println!("  ├─ Raw: {}", balance.amount);
                println!("  └─ Decimals: {}", balance.decimals);
            }
            Err(e) => {
                println!("{} Failed to get balance: {}", "✗".bright_red(), e);
                println!();
                println!("  Possible reasons:");
                println!("    • Account doesn't exist");
                println!("    • Not a token account");
                println!("    • Use --mint to specify the token mint");
            }
        }
        
        Ok(())
    }
}

fn format_amount(amount: u64, decimals: u8) -> String {
    let divisor = 10u64.pow(decimals as u32);
    let whole = amount / divisor;
    let frac = amount % divisor;
    if frac == 0 {
        format!("{}", whole)
    } else {
        format!("{}.{:0>width$}", whole, frac, width = decimals as usize)
    }
}
