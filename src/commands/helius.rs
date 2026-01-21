use clap::{Args, Subcommand};
use colored::Colorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::error::Result;
use crate::config::AppConfig;

/// Helius API commands (DAS, Priority Fees, Webhooks)
#[derive(Args)]
pub struct HeliusCommand {
    #[command(subcommand)]
    pub action: HeliusAction,
}

#[derive(Subcommand)]
pub enum HeliusAction {
    /// Query assets using DAS API
    Assets {
        #[command(subcommand)]
        query: AssetsQuery,
    },
    
    /// Get priority fee estimates
    Fees {
        /// Transaction to estimate fees for (base64 encoded)
        #[arg(short, long)]
        transaction: Option<String>,
        
        /// Account keys to check (comma-separated)
        #[arg(short, long)]
        accounts: Option<String>,
        
        /// Priority level (min, low, medium, high, veryHigh, unsafeMax)
        #[arg(short, long, default_value = "medium")]
        level: String,
    },
    
    /// Webhook management
    Webhooks {
        #[command(subcommand)]
        action: WebhookAction,
    },
    
    /// Enhanced transaction sending with automatic priority fees
    Send {
        /// Base64-encoded transaction
        #[arg(short, long)]
        transaction: String,
        
        /// Skip preflight checks
        #[arg(long)]
        skip_preflight: bool,
        
        /// Auto-set priority fees
        #[arg(long, default_value = "true")]
        auto_fees: bool,
    },
    
    /// Show Helius API information and setup
    Info,
    
    /// Configure Helius API key
    Config {
        /// Helius API key
        #[arg(long)]
        api_key: Option<String>,
        
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
}

#[derive(Subcommand)]
pub enum AssetsQuery {
    /// Get asset by ID
    Get {
        /// Asset ID (mint address)
        #[arg(short, long)]
        id: String,
    },
    
    /// Get assets by owner
    ByOwner {
        /// Owner address
        #[arg(short, long)]
        owner: String,
        
        /// Include fungible tokens
        #[arg(long)]
        fungible: bool,
        
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        
        /// Limit per page
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    
    /// Search assets with filters
    Search {
        /// Owner address (optional)
        #[arg(short, long)]
        owner: Option<String>,
        
        /// Collection address (optional)
        #[arg(short, long)]
        collection: Option<String>,
        
        /// Creator address (optional)
        #[arg(long)]
        creator: Option<String>,
        
        /// Include compressed NFTs
        #[arg(long)]
        compressed: bool,
    },
    
    /// Get token accounts
    Tokens {
        /// Owner address
        #[arg(short, long)]
        owner: String,
        
        /// Specific mint (optional)
        #[arg(short, long)]
        mint: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum WebhookAction {
    /// List webhooks
    List,
    
    /// Create a new webhook
    Create {
        /// Webhook URL endpoint
        #[arg(short, long)]
        url: String,
        
        /// Webhook type (enhanced, raw, discord)
        #[arg(short = 't', long, default_value = "enhanced")]
        webhook_type: String,
        
        /// Account addresses to monitor (comma-separated)
        #[arg(short, long)]
        accounts: Option<String>,
        
        /// Transaction types to monitor (comma-separated)
        #[arg(long)]
        transaction_types: Option<String>,
    },
    
    /// Delete a webhook
    Delete {
        /// Webhook ID
        #[arg(short, long)]
        id: String,
    },
    
    /// Show webhook setup guide
    Guide,
}

// DAS API response types - kept for future type-safe parsing
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct DasAsset {
    id: String,
    content: Option<AssetContent>,
    ownership: Option<AssetOwnership>,
    compression: Option<AssetCompression>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct AssetContent {
    json_uri: Option<String>,
    metadata: Option<AssetMetadata>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct AssetMetadata {
    name: Option<String>,
    symbol: Option<String>,
    description: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct AssetOwnership {
    owner: String,
    frozen: bool,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct AssetCompression {
    compressed: bool,
    tree: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct PriorityFeeResponse {
    #[serde(rename = "priorityFeeEstimate")]
    priority_fee_estimate: Option<f64>,
    #[serde(rename = "priorityFeeLevels")]
    priority_fee_levels: Option<PriorityFeeLevels>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct PriorityFeeLevels {
    min: Option<f64>,
    low: Option<f64>,
    medium: Option<f64>,
    high: Option<f64>,
    #[serde(rename = "veryHigh")]
    very_high: Option<f64>,
    #[serde(rename = "unsafeMax")]
    unsafe_max: Option<f64>,
}

impl HeliusCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            HeliusAction::Assets { query } => self.query_assets(query).await,
            HeliusAction::Fees { transaction, accounts, level } => {
                self.get_priority_fees(transaction.clone(), accounts.clone(), level).await
            }
            HeliusAction::Webhooks { action } => self.handle_webhooks(action).await,
            HeliusAction::Send { transaction, skip_preflight, auto_fees } => {
                self.send_transaction(transaction, *skip_preflight, *auto_fees).await
            }
            HeliusAction::Info => self.show_info(),
            HeliusAction::Config { api_key, show } => self.configure(api_key.clone(), *show),
        }
    }
    
    fn get_helius_url(&self) -> Option<String> {
        if let Ok(config) = AppConfig::load() {
            if let Some(ref api_key) = config.rpc.helius_api_key {
                let network = if config.network == "mainnet" { "mainnet" } else { "devnet" };
                return Some(format!("https://{}.helius-rpc.com/?api-key={}", network, api_key));
            }
        }
        None
    }
    
    fn check_api_key(&self) -> bool {
        if self.get_helius_url().is_none() {
            println!("{} Helius API key not configured!", "✗".bright_red());
            println!();
            println!("  {}:", "Setup".bright_white());
            println!("    1. Get API key from https://helius.xyz");
            println!("    2. Run: solprivacy config helius --api-key YOUR_KEY");
            println!();
            return false;
        }
        true
    }
    
    async fn query_assets(&self, query: &AssetsQuery) -> Result<()> {
        if !self.check_api_key() {
            return Ok(());
        }
        
        let url = self.get_helius_url().unwrap();
        let client = Client::new();
        
        match query {
            AssetsQuery::Get { id } => {
                println!("{} Get Asset by ID", "→".bright_cyan());
                println!("{}", "─".repeat(60).bright_black());
                println!();
                println!("  Asset ID: {}", id);
                println!();
                
                let body = json!({
                    "jsonrpc": "2.0",
                    "id": "solprivacy",
                    "method": "getAsset",
                    "params": {
                        "id": id
                    }
                });
                
                self.make_das_request(&client, &url, body).await?;
            }
            
            AssetsQuery::ByOwner { owner, fungible, page, limit } => {
                println!("{} Get Assets by Owner", "→".bright_cyan());
                println!("{}", "─".repeat(60).bright_black());
                println!();
                println!("  Owner: {}", owner);
                println!("  Include Fungible: {}", fungible);
                println!("  Page: {}, Limit: {}", page, limit);
                println!();
                
                let body = json!({
                    "jsonrpc": "2.0",
                    "id": "solprivacy",
                    "method": "getAssetsByOwner",
                    "params": {
                        "ownerAddress": owner,
                        "page": page,
                        "limit": limit,
                        "displayOptions": {
                            "showFungible": fungible
                        }
                    }
                });
                
                self.make_das_request(&client, &url, body).await?;
            }
            
            AssetsQuery::Search { owner, collection, creator, compressed } => {
                println!("{} Search Assets", "→".bright_cyan());
                println!("{}", "─".repeat(60).bright_black());
                println!();
                
                let mut params = json!({
                    "page": 1,
                    "limit": 100
                });
                
                if let Some(o) = owner {
                    params["ownerAddress"] = json!(o);
                    println!("  Owner: {}", o);
                }
                if let Some(c) = collection {
                    params["grouping"] = json!(["collection", c]);
                    println!("  Collection: {}", c);
                }
                if let Some(cr) = creator {
                    params["creatorAddress"] = json!(cr);
                    println!("  Creator: {}", cr);
                }
                if *compressed {
                    params["compressed"] = json!(true);
                    println!("  Compressed: true");
                }
                println!();
                
                let body = json!({
                    "jsonrpc": "2.0",
                    "id": "solprivacy",
                    "method": "searchAssets",
                    "params": params
                });
                
                self.make_das_request(&client, &url, body).await?;
            }
            
            AssetsQuery::Tokens { owner, mint } => {
                println!("{} Get Token Accounts", "→".bright_cyan());
                println!("{}", "─".repeat(60).bright_black());
                println!();
                println!("  Owner: {}", owner);
                if let Some(m) = mint {
                    println!("  Mint: {}", m);
                }
                println!();
                
                let mut params = json!({
                    "owner": owner
                });
                
                if let Some(m) = mint {
                    params["mint"] = json!(m);
                }
                
                let body = json!({
                    "jsonrpc": "2.0",
                    "id": "solprivacy",
                    "method": "getTokenAccounts",
                    "params": params
                });
                
                self.make_das_request(&client, &url, body).await?;
            }
        }
        
        Ok(())
    }
    
    async fn make_das_request(&self, client: &Client, url: &str, body: serde_json::Value) -> Result<()> {
        println!("  {} Querying DAS API...", "→".bright_cyan());
        
        match client.post(url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await 
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(data) => {
                            println!("{} Success!", "✓".bright_green());
                            println!();
                            
                            if let Some(result) = data.get("result") {
                                // Pretty print the result
                                if let Some(items) = result.get("items") {
                                    if let Some(arr) = items.as_array() {
                                        println!("  Found {} asset(s)", arr.len());
                                        println!();
                                        
                                        for (i, item) in arr.iter().take(5).enumerate() {
                                            self.display_asset(i + 1, item);
                                        }
                                        
                                        if arr.len() > 5 {
                                            println!("  ... and {} more", arr.len() - 5);
                                        }
                                    }
                                } else {
                                    // Single asset
                                    self.display_asset(1, result);
                                }
                            }
                        }
                        Err(e) => {
                            println!("{} Failed to parse response: {}", "✗".bright_red(), e);
                        }
                    }
                } else {
                    println!("{} Request failed: {}", "✗".bright_red(), response.status());
                }
            }
            Err(e) => {
                println!("{} Network error: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    fn display_asset(&self, index: usize, asset: &serde_json::Value) {
        println!("  {}:", format!("Asset {}", index).bright_white());
        
        if let Some(id) = asset.get("id").and_then(|v| v.as_str()) {
            println!("    ID: {}", id.bright_black());
        }
        
        if let Some(content) = asset.get("content") {
            if let Some(metadata) = content.get("metadata") {
                if let Some(name) = metadata.get("name").and_then(|v| v.as_str()) {
                    println!("    Name: {}", name.bright_white());
                }
                if let Some(symbol) = metadata.get("symbol").and_then(|v| v.as_str()) {
                    println!("    Symbol: {}", symbol);
                }
            }
        }
        
        if let Some(ownership) = asset.get("ownership") {
            if let Some(owner) = ownership.get("owner").and_then(|v| v.as_str()) {
                println!("    Owner: {}", 
                    if owner.len() > 20 {
                        format!("{}...{}", &owner[..8], &owner[owner.len()-8..])
                    } else {
                        owner.to_string()
                    }
                );
            }
        }
        
        if let Some(compression) = asset.get("compression") {
            if let Some(compressed) = compression.get("compressed").and_then(|v| v.as_bool()) {
                if compressed {
                    println!("    Compressed: {}", "Yes".bright_cyan());
                }
            }
        }
        
        println!();
    }
    
    async fn get_priority_fees(&self, transaction: Option<String>, accounts: Option<String>, level: &str) -> Result<()> {
        println!("{} Priority Fee Estimation", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        if !self.check_api_key() {
            return Ok(());
        }
        
        let url = self.get_helius_url().unwrap();
        let client = Client::new();
        
        let params = if let Some(tx) = transaction {
            println!("  Method: Serialized Transaction (Recommended)");
            println!("  Transaction: {}...", &tx[..32.min(tx.len())]);
            json!({
                "transaction": tx,
                "options": {
                    "recommended": true,
                    "priorityLevel": level.to_uppercase()
                }
            })
        } else if let Some(accts) = accounts {
            let account_list: Vec<&str> = accts.split(',').map(|s| s.trim()).collect();
            println!("  Method: Account Keys");
            println!("  Accounts: {} address(es)", account_list.len());
            json!({
                "accountKeys": account_list,
                "options": {
                    "recommended": true,
                    "priorityLevel": level.to_uppercase()
                }
            })
        } else {
            println!("  Method: Global Estimate");
            json!({
                "options": {
                    "recommended": true,
                    "includeAllPriorityFeeLevels": true
                }
            })
        };
        
        println!("  Priority Level: {}", level.bright_white());
        println!();
        
        let body = json!({
            "jsonrpc": "2.0",
            "id": "solprivacy",
            "method": "getPriorityFeeEstimate",
            "params": [params]
        });
        
        println!("  {} Fetching priority fees...", "→".bright_cyan());
        
        match client.post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(data) => {
                            println!("{} Fee estimates received!", "✓".bright_green());
                            println!();
                            
                            if let Some(result) = data.get("result") {
                                if let Some(estimate) = result.get("priorityFeeEstimate") {
                                    println!("  {}:", "Recommended Fee".bright_white());
                                    println!("    {} microlamports/CU", 
                                        format!("{:.0}", estimate.as_f64().unwrap_or(0.0)).bright_green()
                                    );
                                    println!();
                                }
                                
                                if let Some(levels) = result.get("priorityFeeLevels") {
                                    println!("  {}:", "All Priority Levels".bright_white());
                                    
                                    if let Some(min) = levels.get("min") {
                                        println!("    Min:      {:>10.0} microlamports/CU", min.as_f64().unwrap_or(0.0));
                                    }
                                    if let Some(low) = levels.get("low") {
                                        println!("    Low:      {:>10.0} microlamports/CU", low.as_f64().unwrap_or(0.0));
                                    }
                                    if let Some(medium) = levels.get("medium") {
                                        println!("    Medium:   {:>10.0} microlamports/CU {}", 
                                            medium.as_f64().unwrap_or(0.0),
                                            "(recommended)".bright_green()
                                        );
                                    }
                                    if let Some(high) = levels.get("high") {
                                        println!("    High:     {:>10.0} microlamports/CU", high.as_f64().unwrap_or(0.0));
                                    }
                                    if let Some(very_high) = levels.get("veryHigh") {
                                        println!("    VeryHigh: {:>10.0} microlamports/CU", very_high.as_f64().unwrap_or(0.0));
                                    }
                                    if let Some(unsafe_max) = levels.get("unsafeMax") {
                                        println!("    UnsafeMax:{:>10.0} microlamports/CU {}", 
                                            unsafe_max.as_f64().unwrap_or(0.0),
                                            "(caution)".bright_yellow()
                                        );
                                    }
                                }
                            }
                            
                            println!();
                            println!("  {}:", "Usage".bright_white());
                            println!("    Add ComputeBudgetInstruction::set_compute_unit_price()");
                            println!("    with the recommended fee to your transaction.");
                        }
                        Err(e) => {
                            println!("{} Failed to parse response: {}", "✗".bright_red(), e);
                        }
                    }
                } else {
                    println!("{} Request failed: {}", "✗".bright_red(), response.status());
                }
            }
            Err(e) => {
                println!("{} Network error: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    async fn handle_webhooks(&self, action: &WebhookAction) -> Result<()> {
        match action {
            WebhookAction::List => self.list_webhooks().await,
            WebhookAction::Create { url, webhook_type, accounts, transaction_types } => {
                self.create_webhook(url, webhook_type, accounts.clone(), transaction_types.clone()).await
            }
            WebhookAction::Delete { id } => self.delete_webhook(id).await,
            WebhookAction::Guide => self.show_webhook_guide(),
        }
    }
    
    async fn list_webhooks(&self) -> Result<()> {
        println!("{} List Webhooks", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        if !self.check_api_key() {
            return Ok(());
        }
        
        println!("  {}:", "Note".bright_yellow());
        println!("    Webhook management requires the Helius REST API.");
        println!("    Use the Helius dashboard for full webhook management:");
        println!("    https://dev.helius.xyz/dashboard/webhooks");
        println!();
        println!("  Or use the API directly:");
        println!("    GET https://api.helius.xyz/v0/webhooks?api-key=YOUR_KEY");
        
        Ok(())
    }
    
    async fn create_webhook(&self, url: &str, webhook_type: &str, accounts: Option<String>, transaction_types: Option<String>) -> Result<()> {
        println!("{} Create Webhook", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  URL: {}", url);
        println!("  Type: {}", webhook_type);
        if let Some(ref accts) = accounts {
            println!("  Accounts: {}", accts);
        }
        if let Some(ref types) = transaction_types {
            println!("  Transaction Types: {}", types);
        }
        println!();
        
        println!("  {}:", "Note".bright_yellow());
        println!("    Webhook creation requires the Helius REST API.");
        println!();
        println!("  {}:", "cURL Example".bright_white());
        println!(r#"    curl -X POST 'https://api.helius.xyz/v0/webhooks?api-key=YOUR_KEY' \
      -H 'Content-Type: application/json' \
      -d '{{
        "webhookURL": "{}",
        "webhookType": "{}",
        "accountAddresses": [{}],
        "transactionTypes": ["Any"]
      }}'"#, 
            url, 
            webhook_type,
            accounts.clone().unwrap_or_default()
        );
        println!();
        println!("  Or use the dashboard: https://dev.helius.xyz/dashboard/webhooks");
        
        Ok(())
    }
    
    async fn delete_webhook(&self, id: &str) -> Result<()> {
        println!("{} Delete Webhook", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  Webhook ID: {}", id);
        println!();
        
        println!("  {}:", "cURL Example".bright_white());
        println!("    curl -X DELETE 'https://api.helius.xyz/v0/webhooks/{}?api-key=YOUR_KEY'", id);
        
        Ok(())
    }
    
    fn show_webhook_guide(&self) -> Result<()> {
        println!("{} Helius Webhooks Guide", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  {}:", "What are Webhooks?".bright_white());
        println!("    Real-time notifications when transactions occur.");
        println!("    No polling required - events pushed to your endpoint.");
        println!();
        
        println!("  {}:", "Webhook Types".bright_white());
        println!("    • {} - Parsed, enriched transaction data", "enhanced".bright_cyan());
        println!("    • {} - Raw transaction data", "raw".bright_cyan());
        println!("    • {} - Direct Discord integration", "discord".bright_cyan());
        println!();
        
        println!("  {}:", "Transaction Types".bright_white());
        println!("    NFT_SALE, NFT_LISTING, NFT_BID, TOKEN_TRANSFER,");
        println!("    SWAP, STAKE, UNSTAKE, BURN, MINT, and more...");
        println!();
        
        println!("  {}:", "Setup Steps".bright_white());
        println!("    1. Create an endpoint to receive webhooks");
        println!("    2. Register webhook via API or dashboard");
        println!("    3. Handle incoming POST requests");
        println!();
        
        println!("  {}:", "Example Endpoint (Express.js)".bright_cyan());
        println!(r#"    app.post('/webhook', (req, res) => {{
      const events = req.body;
      for (const event of events) {{
        console.log('Transaction:', event.signature);
        console.log('Type:', event.type);
        // Process event...
      }}
      res.status(200).send('OK');
    }});"#);
        println!();
        
        println!("  {}:", "Dashboard".bright_white());
        println!("    https://dev.helius.xyz/dashboard/webhooks");
        println!();
        
        println!("  {}:", "Use Cases".bright_white());
        println!("    • Monitor program activity");
        println!("    • Track NFT sales in real-time");
        println!("    • Alert on large transfers");
        println!("    • Trigger automations");
        
        Ok(())
    }
    
    async fn send_transaction(&self, transaction: &str, skip_preflight: bool, auto_fees: bool) -> Result<()> {
        println!("{} Send Transaction with Smart Fees", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        if !self.check_api_key() {
            return Ok(());
        }
        
        println!("  Transaction: {}...", &transaction[..32.min(transaction.len())]);
        println!("  Skip Preflight: {}", skip_preflight);
        println!("  Auto Fees: {}", auto_fees);
        println!();
        
        println!("  {}:", "Note".bright_yellow());
        println!("    Full transaction sending with auto-fees requires");
        println!("    transaction modification. Use the Helius SDK:");
        println!();
        println!("  {}:", "TypeScript Example".bright_cyan());
        println!(r#"    import {{ Helius }} from 'helius-sdk';
    
    const helius = new Helius('YOUR_API_KEY');
    
    // Automatically adds priority fees
    const sig = await helius.rpc.sendSmartTransaction(
      instructions,
      signers,
      [], // lookup tables
      {{ skipPreflight: {} }}
    );
    
    console.log('Signature:', sig);"#, skip_preflight);
        
        Ok(())
    }
    
    fn show_info(&self) -> Result<()> {
        println!("{} Helius API - Enhanced Solana Infrastructure", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  {}:", "What is Helius?".bright_white());
        println!("    Helius provides enhanced RPC, APIs, and webhooks");
        println!("    for Solana developers. Key features include:");
        println!();
        
        println!("  {}:", "DAS API (Digital Asset Standard)".bright_white());
        println!("    • Unified NFT and token queries");
        println!("    • Compressed NFT support");
        println!("    • Rich metadata retrieval");
        println!("    • Collection and creator filtering");
        println!();
        
        println!("  {}:", "Priority Fee API".bright_white());
        println!("    • Real-time fee estimation");
        println!("    • Transaction-specific recommendations");
        println!("    • Multiple priority levels");
        println!();
        
        println!("  {}:", "Webhooks".bright_white());
        println!("    • Real-time transaction notifications");
        println!("    • Enhanced (parsed) or raw format");
        println!("    • Discord integration");
        println!();
        
        println!("  {}:", "Quick Start".bright_white());
        println!("    # Configure API key");
        println!("    solprivacy config helius --api-key YOUR_KEY");
        println!();
        println!("    # Query assets");
        println!("    solprivacy helius assets by-owner --owner <PUBKEY>");
        println!();
        println!("    # Get priority fees");
        println!("    solprivacy helius fees --level medium");
        println!();
        
        // Check current status
        println!("  {}:", "Current Status".bright_white());
        if let Ok(config) = AppConfig::load() {
            if config.rpc.helius_api_key.is_some() {
                println!("    {} API Key: Configured", "✓".bright_green());
                println!("    {} Network: {}", "✓".bright_green(), config.network);
            } else {
                println!("    {} API Key: Not configured", "✗".bright_red());
            }
        } else {
            println!("    {} API Key: Not configured", "✗".bright_red());
        }
        println!();
        
        println!("  {}:", "Resources".bright_white());
        println!("    Website: https://helius.xyz");
        println!("    Docs: https://docs.helius.dev");
        println!("    Dashboard: https://dev.helius.xyz");
        
        Ok(())
    }
    
    fn configure(&self, api_key: Option<String>, show: bool) -> Result<()> {
        println!("{} Helius API Configuration", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        if show {
            if let Ok(config) = AppConfig::load() {
                println!("  {}:", "Current Configuration".bright_white());
                if let Some(ref key) = config.rpc.helius_api_key {
                    println!("    API Key: {}...", &key[..8.min(key.len())]);
                    println!("    Network: {}", config.network);
                    println!("    RPC URL: {}", config.get_rpc_url());
                } else {
                    println!("    API Key: {}", "Not configured".bright_red());
                }
            }
            return Ok(());
        }
        
        if let Some(key) = api_key {
            println!("  {}:", "To configure Helius".bright_white());
            println!("    Run: solprivacy config helius --api-key {}", key);
        } else {
            println!("  {}:", "Usage".bright_white());
            println!("    solprivacy config helius --api-key <YOUR_KEY>");
            println!("    solprivacy helius config --show");
            println!();
            println!("  {}:", "Get an API Key".bright_white());
            println!("    1. Sign up at https://helius.xyz");
            println!("    2. Create a new API key");
            println!("    3. Configure: solprivacy config helius --api-key YOUR_KEY");
        }
        
        Ok(())
    }
}
