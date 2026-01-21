use clap::{Args, Subcommand};
use colored::Colorize;
use std::process::Command;
use crate::error::Result;
use crate::config::AppConfig;

// Light Protocol native SDK imports
use light_client::{
    rpc::{LightClient, LightClientConfig, Rpc},
    indexer::{Indexer, IndexerRpcConfig, RetryConfig, GetCompressedTokenAccountsByOwnerOrDelegateOptions},
};
use solana_pubkey::Pubkey;  // Use v2 Pubkey for light-client compatibility
use std::str::FromStr;

/// Light Protocol ZK Compression commands
#[derive(Args)]
pub struct LightCommand {
    #[command(subcommand)]
    pub action: LightAction,
}

#[derive(Subcommand)]
pub enum LightAction {
    /// Setup Light Protocol CLI and dependencies
    Setup,
    
    /// Configure RPC endpoint for Light Protocol
    Config {
        /// Photon RPC URL (e.g., https://devnet.helius-rpc.com/?api-key=YOUR_KEY)
        #[arg(short, long)]
        rpc_url: Option<String>,
        
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
    
    /// Query compressed accounts for an owner (native SDK)
    Accounts {
        /// Owner public key
        #[arg(short, long)]
        owner: String,
        
        /// RPC URL override
        #[arg(long)]
        rpc_url: Option<String>,
    },
    
    /// Query compressed token balances for an owner (native SDK)
    Balance {
        /// Owner public key
        #[arg(short, long)]
        owner: String,
        
        /// Filter by mint address
        #[arg(short, long)]
        mint: Option<String>,
        
        /// RPC URL override
        #[arg(long)]
        rpc_url: Option<String>,
    },
    
    /// Check indexer health and status (native SDK)
    Health {
        /// RPC URL override
        #[arg(long)]
        rpc_url: Option<String>,
    },
    
    /// Create a compressed token mint
    CreateMint {
        /// Token decimals
        #[arg(short, long, default_value = "9")]
        decimals: u8,
    },
    
    /// Mint compressed tokens
    Mint {
        /// Mint address
        #[arg(short, long)]
        mint: String,
        
        /// Recipient address
        #[arg(short, long)]
        to: String,
        
        /// Amount to mint
        #[arg(short, long)]
        amount: u64,
    },
    
    /// Transfer compressed tokens
    Transfer {
        /// Mint address
        #[arg(short, long)]
        mint: String,
        
        /// Recipient address
        #[arg(short, long)]
        to: String,
        
        /// Amount to transfer
        #[arg(short, long)]
        amount: u64,
    },
    
    /// Compress SOL
    CompressSol {
        /// Amount in SOL
        #[arg(short, long)]
        amount: f64,
    },
    
    /// Decompress SOL
    DecompressSol {
        /// Amount in SOL
        #[arg(short, long)]
        amount: f64,
    },
    
    /// Compare costs between regular and compressed accounts
    Compare {
        /// Number of accounts to compare
        #[arg(short, long, default_value = "100")]
        count: u64,
        
        /// Account data size in bytes
        #[arg(short, long, default_value = "100")]
        size: u64,
    },
    
    /// Show Light Protocol information and resources
    Info,
    
    /// Show complete ZK compression workflow
    Workflow,
}

impl LightCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            LightAction::Setup => self.setup().await,
            LightAction::Config { rpc_url, show } => self.configure(rpc_url.clone(), *show).await,
            LightAction::Accounts { owner, rpc_url } => self.query_accounts(owner, rpc_url.clone()).await,
            LightAction::Balance { owner, mint, rpc_url } => self.query_balance(owner, mint.clone(), rpc_url.clone()).await,
            LightAction::Health { rpc_url } => self.check_health(rpc_url.clone()).await,
            LightAction::CreateMint { decimals } => self.create_mint(*decimals).await,
            LightAction::Mint { mint, to, amount } => self.mint_tokens(mint, to, *amount).await,
            LightAction::Transfer { mint, to, amount } => self.transfer(mint, to, *amount).await,
            LightAction::CompressSol { amount } => self.compress_sol(*amount).await,
            LightAction::DecompressSol { amount } => self.decompress_sol(*amount).await,
            LightAction::Compare { count, size } => self.compare_costs(*count, *size).await,
            LightAction::Info => self.show_info().await,
            LightAction::Workflow => self.show_workflow().await,
        }
    }
    
    fn is_light_installed(&self) -> bool {
        Command::new("light").arg("--version").output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    fn get_rpc_url(&self, override_url: Option<String>) -> String {
        if let Some(url) = override_url {
            return url;
        }
        
        // Try to get from app config
        if let Ok(config) = AppConfig::load() {
            // Check for photon-specific URL first
            if let Some(ref photon_url) = config.rpc.photon_url {
                return photon_url.clone();
            }
            // Otherwise use the general RPC URL
            return config.get_rpc_url();
        }
        
        // Default to local
        "http://127.0.0.1:8899".to_string()
    }
    
    async fn configure(&self, rpc_url: Option<String>, show: bool) -> Result<()> {
        println!("{} Light Protocol Configuration", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if show {
            if let Ok(config) = AppConfig::load() {
                if let Some(ref photon_url) = config.rpc.photon_url {
                    println!("  {} Photon RPC URL: {}", "✓".bright_green(), photon_url);
                } else {
                    println!("  {} Photon RPC URL: {} (using default RPC)", "ℹ".bright_blue(), config.get_rpc_url());
                }
            } else {
                println!("  {} Photon RPC URL: http://127.0.0.1:8899 (default)", "ℹ".bright_blue());
            }
            return Ok(());
        }
        
        if let Some(url) = rpc_url {
            println!("  Setting Photon RPC URL: {}", url.bright_white());
            println!();
            println!("  {}:", "Note".bright_yellow());
            println!("    To persist this setting, run:");
            println!("    solprivacy config custom --url {}", url);
            println!();
            println!("  {}:", "Recommended Providers".bright_white());
            println!("    • Helius: https://devnet.helius-rpc.com/?api-key=YOUR_KEY");
            println!("    • Triton: https://zk-compression-devnet.triton.one/?api-key=YOUR_KEY");
            println!("    • Local: http://127.0.0.1:8899 (with light test-validator)");
        } else {
            println!("  {}:", "Usage".bright_white());
            println!("    solprivacy light config --rpc-url <URL>");
            println!("    solprivacy light config --show");
        }
        
        Ok(())
    }
    
    async fn query_accounts(&self, owner: &str, rpc_url: Option<String>) -> Result<()> {
        println!("{} Query Compressed Accounts (Native SDK)", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        let rpc_url = self.get_rpc_url(rpc_url);
        println!("  RPC: {}", rpc_url.bright_black());
        println!("  Owner: {}", owner);
        println!();
        
        // Parse owner pubkey
        let owner_pubkey = match Pubkey::from_str(owner) {
            Ok(pk) => pk,
            Err(e) => {
                println!("{} Invalid owner pubkey: {}", "✗".bright_red(), e);
                return Ok(());
            }
        };
        
        // Connect to Light Protocol RPC
        println!("  {} Connecting to Photon RPC...", "→".bright_cyan());
        
        let config = LightClientConfig::new(rpc_url.clone(), Some(rpc_url.clone()), None);
        let rpc: LightClient = match LightClient::new(config).await {
            Ok(r) => r,
            Err(e) => {
                println!("{} Failed to connect: {}", "✗".bright_red(), e);
                println!();
                println!("  {}:", "Troubleshooting".bright_yellow());
                println!("    • Ensure you're using a Photon-compatible RPC endpoint");
                println!("    • For devnet: https://devnet.helius-rpc.com/?api-key=YOUR_KEY");
                println!("    • For local: Run 'light test-validator' first");
                return Ok(());
            }
        };
        
        // Query compressed accounts
        println!("  {} Querying compressed accounts...", "→".bright_cyan());
        
        let slot = rpc.get_slot().await.unwrap_or(0);
        let indexer_config = IndexerRpcConfig {
            slot,
            retry_config: RetryConfig::default(),
        };
        
        match rpc.get_compressed_accounts_by_owner(&owner_pubkey, None, Some(indexer_config)).await {
            Ok(response) => {
                let accounts = response.value.items;
                println!();
                println!("{} Found {} compressed account(s)", "✓".bright_green(), accounts.len());
                println!();
                
                if accounts.is_empty() {
                    println!("  No compressed accounts found for this owner.");
                    println!();
                    println!("  {}:", "To create compressed accounts".bright_white());
                    println!("    1. Compress SOL: solprivacy light compress-sol --amount 0.1");
                    println!("    2. Create compressed token: solprivacy light create-mint");
                } else {
                    for (i, account) in accounts.iter().enumerate() {
                        println!("  {}:", format!("Account {}", i + 1).bright_white());
                        println!("    Hash: {}", bs58::encode(&account.hash).into_string().bright_black());
                        println!("    Lamports: {}", account.lamports);
                        if let Some(data) = &account.data {
                            println!("    Data size: {} bytes", data.data.len());
                            println!("    Discriminator: {:?}", data.discriminator);
                        }
                        println!();
                    }
                }
            }
            Err(e) => {
                println!("{} Query failed: {}", "✗".bright_red(), e);
                println!();
                println!("  This may indicate:");
                println!("    • The RPC endpoint doesn't support ZK Compression");
                println!("    • Network connectivity issues");
                println!("    • The owner has no compressed accounts");
            }
        }
        
        Ok(())
    }
    
    async fn query_balance(&self, owner: &str, mint: Option<String>, rpc_url: Option<String>) -> Result<()> {
        println!("{} Query Compressed Token Balance (Native SDK)", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        let rpc_url = self.get_rpc_url(rpc_url);
        println!("  RPC: {}", rpc_url.bright_black());
        println!("  Owner: {}", owner);
        if let Some(ref m) = mint {
            println!("  Mint: {}", m);
        }
        println!();
        
        // Parse owner pubkey
        let owner_pubkey = match Pubkey::from_str(owner) {
            Ok(pk) => pk,
            Err(e) => {
                println!("{} Invalid owner pubkey: {}", "✗".bright_red(), e);
                return Ok(());
            }
        };
        
        // Parse mint if provided
        let mint_pubkey = if let Some(ref m) = mint {
            match Pubkey::from_str(m) {
                Ok(pk) => Some(pk),
                Err(e) => {
                    println!("{} Invalid mint pubkey: {}", "✗".bright_red(), e);
                    return Ok(());
                }
            }
        } else {
            None
        };
        
        // Connect to Light Protocol RPC
        println!("  {} Connecting to Photon RPC...", "→".bright_cyan());
        
        let config = LightClientConfig::new(rpc_url.clone(), Some(rpc_url.clone()), None);
        let rpc: LightClient = match LightClient::new(config).await {
            Ok(r) => r,
            Err(e) => {
                println!("{} Failed to connect: {}", "✗".bright_red(), e);
                return Ok(());
            }
        };
        
        // Query token balances
        println!("  {} Querying compressed token balances...", "→".bright_cyan());
        
        let slot = rpc.get_slot().await.unwrap_or(0);
        let indexer_config = IndexerRpcConfig {
            slot,
            retry_config: RetryConfig::default(),
        };
        
        let options = mint_pubkey.map(|m| GetCompressedTokenAccountsByOwnerOrDelegateOptions::new(Some(m)));
        
        match rpc.get_compressed_token_balances_by_owner_v2(&owner_pubkey, options, Some(indexer_config)).await {
            Ok(response) => {
                let balances = response.value.items;
                println!();
                println!("{} Found {} token balance(s)", "✓".bright_green(), balances.len());
                println!();
                
                if balances.is_empty() {
                    println!("  No compressed token balances found.");
                    println!();
                    println!("  {}:", "To get compressed tokens".bright_white());
                    println!("    1. Create a compressed mint: solprivacy light create-mint");
                    println!("    2. Mint tokens: solprivacy light mint --mint <MINT> --to <OWNER> --amount 1000");
                } else {
                    let mut total_lamports: u64 = 0;
                    for balance in &balances {
                        println!("  {}:", format!("Mint: {}", balance.mint).bright_white());
                        println!("    Balance: {}", balance.balance.to_string().bright_green());
                        total_lamports += balance.balance;
                        println!();
                    }
                    
                    println!("  {}:", "Summary".bright_white());
                    println!("    Total token types: {}", balances.len());
                    println!("    Total balance (raw): {}", total_lamports);
                }
            }
            Err(e) => {
                println!("{} Query failed: {}", "✗".bright_red(), e);
            }
        }
        
        // Also query compressed SOL balance
        println!();
        println!("  {} Querying compressed SOL balance...", "→".bright_cyan());
        
        let slot = rpc.get_slot().await.unwrap_or(0);
        let indexer_config = IndexerRpcConfig {
            slot,
            retry_config: RetryConfig::default(),
        };
        
        match rpc.get_compressed_balance_by_owner(&owner_pubkey, Some(indexer_config)).await {
            Ok(response) => {
                let lamports = response.value;
                let sol = lamports as f64 / 1_000_000_000.0;
                println!("  {} Compressed SOL: {} SOL ({} lamports)", 
                    "✓".bright_green(), 
                    format!("{:.9}", sol).bright_green(),
                    lamports
                );
            }
            Err(e) => {
                println!("  {} Could not query SOL balance: {}", "ℹ".bright_blue(), e);
            }
        }
        
        Ok(())
    }
    
    async fn check_health(&self, rpc_url: Option<String>) -> Result<()> {
        println!("{} Light Protocol Indexer Health", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        let rpc_url = self.get_rpc_url(rpc_url);
        println!("  RPC: {}", rpc_url.bright_black());
        println!();
        
        // Connect to Light Protocol RPC
        let config = LightClientConfig::new(rpc_url.clone(), Some(rpc_url), None);
        let rpc: LightClient = match LightClient::new(config).await {
            Ok(r) => r,
            Err(e) => {
                println!("{} Failed to connect: {}", "✗".bright_red(), e);
                return Ok(());
            }
        };
        
        // Check indexer health
        println!("  {} Checking indexer health...", "→".bright_cyan());
        
        match rpc.get_indexer_health(Some(RetryConfig::default())).await {
            Ok(healthy) => {
                if healthy {
                    println!("  {} Indexer is healthy", "✓".bright_green());
                } else {
                    println!("  {} Indexer may be stale", "⚠".bright_yellow());
                }
            }
            Err(e) => {
                println!("  {} Health check failed: {}", "✗".bright_red(), e);
            }
        }
        
        // Get indexer slot
        match rpc.get_indexer_slot(Some(RetryConfig::default())).await {
            Ok(slot) => {
                println!("  {} Indexer slot: {}", "✓".bright_green(), slot);
            }
            Err(e) => {
                println!("  {} Could not get slot: {}", "ℹ".bright_blue(), e);
            }
        }
        
        // Get RPC slot for comparison
        match rpc.get_slot().await {
            Ok(slot) => {
                println!("  {} RPC slot: {}", "✓".bright_green(), slot);
            }
            Err(e) => {
                println!("  {} Could not get RPC slot: {}", "ℹ".bright_blue(), e);
            }
        }
        
        Ok(())
    }
    
    async fn compare_costs(&self, count: u64, size: u64) -> Result<()> {
        println!("{} Cost Comparison: Regular vs Compressed", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        println!("  {}:", "Parameters".bright_white());
        println!("    Account count: {}", count);
        println!("    Data size per account: {} bytes", size);
        println!();
        
        // Regular account costs
        // Rent exemption: (128 + data_size) * 6.96e-6 SOL per byte-epoch
        // Simplified: ~0.00203928 SOL for 0-byte account + ~6.96e-6 per additional byte
        let base_rent = 890_880u64; // lamports for minimum account (0 bytes data)
        let per_byte_rent = 6960u64; // lamports per byte
        let regular_rent_per_account = base_rent + (size * per_byte_rent);
        let regular_total = regular_rent_per_account * count;
        
        // Compressed account costs (approximate)
        // State tree: ~5000 lamports per leaf
        // No rent exemption required
        let compressed_per_account = 5000u64; // approximate cost per compressed account
        let compressed_total = compressed_per_account * count;
        
        let regular_sol = regular_total as f64 / 1_000_000_000.0;
        let compressed_sol = compressed_total as f64 / 1_000_000_000.0;
        let savings = regular_total.saturating_sub(compressed_total);
        let savings_sol = savings as f64 / 1_000_000_000.0;
        let savings_pct = if regular_total > 0 {
            (savings as f64 / regular_total as f64) * 100.0
        } else {
            0.0
        };
        
        println!("  {}:", "Regular Accounts".bright_white());
        println!("    Per account: {} lamports ({:.6} SOL)", regular_rent_per_account, regular_rent_per_account as f64 / 1e9);
        println!("    Total ({} accounts): {} lamports ({:.6} SOL)", count, regular_total, regular_sol);
        println!();
        
        println!("  {}:", "Compressed Accounts".bright_white());
        println!("    Per account: ~{} lamports ({:.6} SOL)", compressed_per_account, compressed_per_account as f64 / 1e9);
        println!("    Total ({} accounts): ~{} lamports ({:.6} SOL)", count, compressed_total, compressed_sol);
        println!();
        
        println!("  {}:", "Savings".bright_green());
        println!("    Total saved: {} lamports ({:.6} SOL)", savings, savings_sol);
        println!("    Percentage: {:.1}%", savings_pct);
        println!();
        
        println!("  {}:", "Cost Multiplier".bright_white());
        if compressed_total > 0 {
            let multiplier = regular_total as f64 / compressed_total as f64;
            println!("    Regular accounts cost {:.0}x more than compressed", multiplier);
        }
        println!();
        
        println!("  {}:", "Use Cases".bright_white());
        println!("    • NFT collections with {} items", count);
        println!("    • Token airdrops to {} recipients", count);
        println!("    • Gaming: {} player inventories", count);
        println!("    • Social: {} user profiles", count);
        
        Ok(())
    }
    
    async fn show_workflow(&self) -> Result<()> {
        println!("{} ZK Compression Complete Workflow", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        println!("  {}:", "Step 1: Setup Environment".bright_white());
        println!("    # Install Light Protocol CLI");
        println!("    npm i -g @lightprotocol/zk-compression-cli");
        println!();
        println!("    # Start local test validator with Light programs");
        println!("    light test-validator");
        println!();
        
        println!("  {}:", "Step 2: Configure RPC".bright_white());
        println!("    # For devnet (recommended for testing)");
        println!("    solprivacy light config --rpc-url https://devnet.helius-rpc.com/?api-key=YOUR_KEY");
        println!();
        println!("    # For local development");
        println!("    solprivacy light config --rpc-url http://127.0.0.1:8899");
        println!();
        
        println!("  {}:", "Step 3: Compress SOL".bright_white());
        println!("    # Convert regular SOL to compressed SOL");
        println!("    solprivacy light compress-sol --amount 0.1");
        println!();
        println!("    # Check compressed balance");
        println!("    solprivacy light balance --owner <YOUR_PUBKEY>");
        println!();
        
        println!("  {}:", "Step 4: Create Compressed Tokens".bright_white());
        println!("    # Create a new compressed token mint");
        println!("    solprivacy light create-mint --decimals 9");
        println!();
        println!("    # Mint compressed tokens");
        println!("    solprivacy light mint --mint <MINT> --to <RECIPIENT> --amount 1000000000");
        println!();
        
        println!("  {}:", "Step 5: Transfer Compressed Tokens".bright_white());
        println!("    # Transfer compressed tokens");
        println!("    solprivacy light transfer --mint <MINT> --to <RECIPIENT> --amount 500000000");
        println!();
        
        println!("  {}:", "Step 6: Decompress (Optional)".bright_white());
        println!("    # Convert compressed SOL back to regular SOL");
        println!("    solprivacy light decompress-sol --amount 0.05");
        println!();
        
        println!("  {}:", "Native SDK Queries".bright_white());
        println!("    # Query compressed accounts");
        println!("    solprivacy light accounts --owner <PUBKEY>");
        println!();
        println!("    # Query token balances");
        println!("    solprivacy light balance --owner <PUBKEY> --mint <MINT>");
        println!();
        println!("    # Check indexer health");
        println!("    solprivacy light health");
        println!();
        
        println!("  {}:", "Cost Analysis".bright_white());
        println!("    # Compare costs for 1000 accounts");
        println!("    solprivacy light compare --count 1000 --size 100");
        println!();
        
        println!("  {}:", "Key Concepts".bright_yellow());
        println!("    • {} - Data stored off-chain, only hash on-chain", "Compressed Accounts".bright_white());
        println!("    • {} - Merkle tree storing account state", "State Tree".bright_white());
        println!("    • {} - ZK proof of account validity", "Validity Proof".bright_white());
        println!("    • {} - RPC with indexer support", "Photon RPC".bright_white());
        
        Ok(())
    }
    
    async fn setup(&self) -> Result<()> {
        println!("{} Light Protocol Setup", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if self.is_light_installed() {
            let version = Command::new("light").arg("--version").output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            println!("{} Light CLI already installed: {}", "✓".bright_green(), version);
            println!();
            println!("  {}:", "Quick Start".bright_white());
            println!("    # Start local test validator");
            println!("    light test-validator");
            println!();
            println!("    # Or use devnet with Helius RPC");
            println!("    solprivacy light config --rpc-url https://devnet.helius-rpc.com/?api-key=YOUR_KEY");
            return Ok(());
        }
        
        println!("  {}:", "Installation".bright_white());
        println!();
        println!("  Light CLI is not installed. Install with:");
        println!();
        println!("    {}", "npm i -g @lightprotocol/zk-compression-cli".bright_cyan());
        println!();
        println!("  Or follow the official guide:");
        println!("    https://www.zkcompression.com/");
        println!();
        println!("  {}:", "Requirements".bright_white());
        println!("    • Node.js 18+");
        println!("    • Solana CLI");
        println!("    • Photon RPC access (for devnet/mainnet)");
        println!();
        println!("  {}:", "RPC Providers".bright_white());
        println!("    • Helius: https://helius.xyz (recommended)");
        println!("    • Triton: https://triton.one");
        
        Ok(())
    }
    
    async fn create_mint(&self, decimals: u8) -> Result<()> {
        println!("{} Create Compressed Token Mint", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if !self.is_light_installed() {
            println!("{} Light CLI not installed!", "✗".bright_red());
            println!("  Run: solprivacy light setup");
            return Ok(());
        }
        
        println!("  Decimals: {}", decimals);
        println!();
        println!("{} Running: light create-mint --decimals {}", "→".bright_cyan(), decimals);
        println!();
        
        let output = Command::new("light")
            .args(["create-mint", "--decimals", &decimals.to_string()])
            .output();
        
        match output {
            Ok(o) => {
                if o.status.success() {
                    println!("{} Mint created!", "✓".bright_green());
                    println!("{}", String::from_utf8_lossy(&o.stdout));
                } else {
                    println!("{} Command failed", "✗".bright_red());
                    println!("{}", String::from_utf8_lossy(&o.stderr));
                }
            }
            Err(e) => {
                println!("{} Failed to run light CLI: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn mint_tokens(&self, mint: &str, to: &str, amount: u64) -> Result<()> {
        println!("{} Mint Compressed Tokens", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if !self.is_light_installed() {
            println!("{} Light CLI not installed!", "✗".bright_red());
            println!("  Run: solprivacy light setup");
            return Ok(());
        }
        
        println!("  Mint: {}", mint);
        println!("  To: {}", to);
        println!("  Amount: {}", amount);
        println!();
        
        let output = Command::new("light")
            .args(["mint-to", "--mint", mint, "--to", to, "--amount", &amount.to_string()])
            .output();
        
        match output {
            Ok(o) => {
                if o.status.success() {
                    println!("{} Tokens minted!", "✓".bright_green());
                    println!("{}", String::from_utf8_lossy(&o.stdout));
                } else {
                    println!("{} Command failed", "✗".bright_red());
                    println!("{}", String::from_utf8_lossy(&o.stderr));
                }
            }
            Err(e) => {
                println!("{} Failed to run light CLI: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn transfer(&self, mint: &str, to: &str, amount: u64) -> Result<()> {
        println!("{} Transfer Compressed Tokens", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if !self.is_light_installed() {
            println!("{} Light CLI not installed!", "✗".bright_red());
            println!("  Run: solprivacy light setup");
            return Ok(());
        }
        
        println!("  Mint: {}", mint);
        println!("  To: {}", to);
        println!("  Amount: {}", amount);
        println!();
        
        let output = Command::new("light")
            .args(["transfer", "--mint", mint, "--to", to, "--amount", &amount.to_string()])
            .output();
        
        match output {
            Ok(o) => {
                if o.status.success() {
                    println!("{} Transfer complete!", "✓".bright_green());
                    println!("{}", String::from_utf8_lossy(&o.stdout));
                } else {
                    println!("{} Command failed", "✗".bright_red());
                    println!("{}", String::from_utf8_lossy(&o.stderr));
                }
            }
            Err(e) => {
                println!("{} Failed to run light CLI: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn compress_sol(&self, amount: f64) -> Result<()> {
        println!("{} Compress SOL", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if !self.is_light_installed() {
            println!("{} Light CLI not installed!", "✗".bright_red());
            println!("  Run: solprivacy light setup");
            return Ok(());
        }
        
        println!("  Amount: {} SOL", amount);
        println!();
        
        let lamports = (amount * 1_000_000_000.0) as u64;
        
        let output = Command::new("light")
            .args(["compress-sol", "--lamports", &lamports.to_string()])
            .output();
        
        match output {
            Ok(o) => {
                if o.status.success() {
                    println!("{} SOL compressed!", "✓".bright_green());
                    println!("{}", String::from_utf8_lossy(&o.stdout));
                    println!();
                    println!("  {}:", "Next Steps".bright_white());
                    println!("    • Check balance: solprivacy light balance --owner <YOUR_PUBKEY>");
                    println!("    • Transfer: solprivacy light transfer --mint <SOL> --to <RECIPIENT> --amount <LAMPORTS>");
                } else {
                    println!("{} Command failed", "✗".bright_red());
                    println!("{}", String::from_utf8_lossy(&o.stderr));
                }
            }
            Err(e) => {
                println!("{} Failed to run light CLI: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn decompress_sol(&self, amount: f64) -> Result<()> {
        println!("{} Decompress SOL", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if !self.is_light_installed() {
            println!("{} Light CLI not installed!", "✗".bright_red());
            println!("  Run: solprivacy light setup");
            return Ok(());
        }
        
        println!("  Amount: {} SOL", amount);
        println!();
        
        let lamports = (amount * 1_000_000_000.0) as u64;
        
        let output = Command::new("light")
            .args(["decompress-sol", "--lamports", &lamports.to_string()])
            .output();
        
        match output {
            Ok(o) => {
                if o.status.success() {
                    println!("{} SOL decompressed!", "✓".bright_green());
                    println!("{}", String::from_utf8_lossy(&o.stdout));
                } else {
                    println!("{} Command failed", "✗".bright_red());
                    println!("{}", String::from_utf8_lossy(&o.stderr));
                }
            }
            Err(e) => {
                println!("{} Failed to run light CLI: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn show_info(&self) -> Result<()> {
        println!("{} Light Protocol - ZK Compression", "→".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        println!("  {}:", "What is ZK Compression?".bright_white());
        println!("    State compression using zero-knowledge proofs.");
        println!("    Store data off-chain with on-chain validity proofs.");
        println!("    Up to 1000x cheaper than regular Solana accounts.");
        println!();
        
        println!("  {}:", "Cost Comparison".bright_white());
        println!("    Regular Token Account:     ~2,000,000 lamports (~0.002 SOL)");
        println!("    Compressed Token Account:      ~5,000 lamports (~0.000005 SOL)");
        println!("    Savings:                   ~400x cheaper!");
        println!();
        
        println!("  {}:", "Use Cases".bright_white());
        println!("    • Large NFT collections (millions of items)");
        println!("    • Token airdrops at massive scale");
        println!("    • Gaming assets and items");
        println!("    • Social tokens and rewards");
        println!("    • DePIN device registries");
        println!();
        
        println!("  {}:", "Architecture".bright_white());
        println!("    • {} - Stores compressed account hashes", "State Trees".bright_cyan());
        println!("    • {} - Off-chain data indexing", "Photon Indexer".bright_cyan());
        println!("    • {} - Proves account validity", "ZK Proofs".bright_cyan());
        println!();
        
        println!("  {}:", "Resources".bright_white());
        println!("    Docs: https://www.zkcompression.com");
        println!("    SDK (JS): @lightprotocol/stateless.js");
        println!("    SDK (Rust): light-sdk, light-client");
        println!("    GitHub: github.com/Lightprotocol/light-protocol");
        println!();
        
        println!("  {}:", "CLI Status".bright_white());
        if self.is_light_installed() {
            let version = Command::new("light").arg("--version").output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            println!("    {} Light CLI installed: {}", "✓".bright_green(), version);
        } else {
            println!("    {} Light CLI not installed", "✗".bright_red());
            println!("    Install: npm i -g @lightprotocol/zk-compression-cli");
        }
        println!();
        
        println!("  {}:", "Native SDK Integration".bright_green());
        println!("    {} light-client v0.18.0 integrated", "✓".bright_green());
        println!("    • Query compressed accounts: solprivacy light accounts --owner <PUBKEY>");
        println!("    • Query token balances: solprivacy light balance --owner <PUBKEY>");
        println!("    • Check indexer health: solprivacy light health");
        
        Ok(())
    }
}
