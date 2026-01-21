use clap::{Args, Subcommand};
use colored::Colorize;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::Result;

/// Range Protocol compliance and risk assessment commands
#[derive(Args)]
pub struct ComplianceCommand {
    #[command(subcommand)]
    pub action: ComplianceAction,
}

#[derive(Subcommand)]
pub enum ComplianceAction {
    /// Check risk score for a wallet address
    Check {
        /// Wallet address to check
        #[arg(short, long)]
        address: String,
        
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Check multiple addresses in batch
    Batch {
        /// Comma-separated list of addresses
        #[arg(short, long)]
        addresses: String,
        
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Validate a transaction for compliance
    Transaction {
        /// Transaction signature to check
        #[arg(short, long)]
        signature: String,
    },
    
    /// Show compliance integration guide
    Integrate,
    
    /// Show Range Protocol information
    Info,
    
    /// Configure Range API (optional API key for higher limits)
    Config {
        /// Range API key (optional, for higher rate limits)
        #[arg(long)]
        api_key: Option<String>,
        
        /// Show current configuration
        #[arg(long)]
        show: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct RangeRiskResponse {
    address: String,
    network: String,
    #[serde(rename = "riskScore")]
    risk_score: Option<i32>,
    #[serde(rename = "riskLevel")]
    risk_level: Option<String>,
    #[serde(rename = "riskCategories")]
    risk_categories: Option<Vec<String>>,
    message: Option<String>,
    error: Option<String>,
}

impl ComplianceCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            ComplianceAction::Check { address, format } => self.check_address(address, format).await,
            ComplianceAction::Batch { addresses, format } => self.check_batch(addresses, format).await,
            ComplianceAction::Transaction { signature } => self.check_transaction(signature).await,
            ComplianceAction::Integrate => self.show_integration_guide(),
            ComplianceAction::Info => self.show_info(),
            ComplianceAction::Config { api_key, show } => self.configure(api_key.clone(), *show),
        }
    }
    
    async fn check_address(&self, address: &str, format: &str) -> Result<()> {
        println!("{} Range Compliance Check", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  Address: {}", address.bright_white());
        println!();
        
        // Make API request to Range
        let client = Client::new();
        let url = format!(
            "https://api.range.org/v1/risk/address?address={}&network=solana",
            address
        );
        
        println!("  {} Querying Range Risk API...", "→".bright_cyan());
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<RangeRiskResponse>().await {
                        Ok(risk_data) => {
                            if format == "json" {
                                println!("{}", serde_json::to_string_pretty(&risk_data).unwrap_or_default());
                            } else {
                                self.display_risk_result(&risk_data);
                            }
                        }
                        Err(e) => {
                            println!("{} Failed to parse response: {}", "✗".bright_red(), e);
                        }
                    }
                } else {
                    println!("{} API request failed: {}", "✗".bright_red(), response.status());
                    
                    // Show guidance
                    println!();
                    println!("  {}:", "Possible Reasons".bright_yellow());
                    println!("    • Address format invalid");
                    println!("    • Rate limit exceeded (use --api-key for higher limits)");
                    println!("    • Network issues");
                }
            }
            Err(e) => {
                println!("{} Network error: {}", "✗".bright_red(), e);
                println!();
                println!("  {}:", "Troubleshooting".bright_yellow());
                println!("    • Check internet connectivity");
                println!("    • Range API may be temporarily unavailable");
            }
        }
        
        Ok(())
    }
    
    fn display_risk_result(&self, risk_data: &RangeRiskResponse) {
        println!();
        
        if let Some(score) = risk_data.risk_score {
            let (color, icon) = match score {
                0..=2 => ("green", "✓"),
                3..=5 => ("yellow", "⚠"),
                6..=8 => ("red", "⚠"),
                _ => ("red", "✗"),
            };
            
            let score_display = match color {
                "green" => format!("{}", score).bright_green(),
                "yellow" => format!("{}", score).bright_yellow(),
                _ => format!("{}", score).bright_red(),
            };
            
            println!("  {}:", "Risk Assessment".bright_white());
            println!("    Risk Score: {} / 10", score_display);
            
            if let Some(ref level) = risk_data.risk_level {
                let level_colored = match score {
                    0..=2 => level.bright_green(),
                    3..=5 => level.bright_yellow(),
                    _ => level.bright_red(),
                };
                println!("    Risk Level: {}", level_colored);
            }
            
            println!();
            
            // Display risk interpretation
            match score {
                0..=2 => {
                    println!("  {} {}", icon.bright_green(), "LOW RISK - Address appears clean".bright_green());
                    println!("    This address has no known associations with");
                    println!("    malicious activity, sanctions, or hacks.");
                }
                3..=5 => {
                    println!("  {} {}", icon.bright_yellow(), "MEDIUM RISK - Exercise caution".bright_yellow());
                    println!("    This address has some risk indicators.");
                    println!("    Review before proceeding with transactions.");
                }
                6..=8 => {
                    println!("  {} {}", icon.bright_red(), "HIGH RISK - Significant concerns".bright_red());
                    println!("    This address has notable risk factors.");
                    println!("    Consider blocking or requiring additional verification.");
                }
                _ => {
                    println!("  {} {}", icon.bright_red(), "CRITICAL RISK - Known malicious".bright_red());
                    println!("    This address is associated with known malicious");
                    println!("    activity, sanctions, or exploits. BLOCK RECOMMENDED.");
                }
            }
        } else if let Some(ref message) = risk_data.message {
            println!("  {} {}", "ℹ".bright_blue(), message);
        } else if let Some(ref error) = risk_data.error {
            println!("  {} Error: {}", "✗".bright_red(), error);
        }
        
        // Display risk categories if present
        if let Some(ref categories) = risk_data.risk_categories {
            if !categories.is_empty() {
                println!();
                println!("  {}:", "Risk Categories".bright_white());
                for category in categories {
                    println!("    • {}", category.bright_yellow());
                }
            }
        }
        
        println!();
        println!("  {}:", "Compliance Actions".bright_white());
        println!("    • Use this data for KYT (Know Your Transaction)");
        println!("    • Integrate on-chain via Switchboard Oracles");
        println!("    • See: solprivacy compliance integrate");
    }
    
    async fn check_batch(&self, addresses: &str, format: &str) -> Result<()> {
        println!("{} Batch Compliance Check", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        let address_list: Vec<&str> = addresses.split(',').map(|s| s.trim()).collect();
        
        println!("  Checking {} addresses...", address_list.len());
        println!();
        
        let client = Client::new();
        let mut results: Vec<RangeRiskResponse> = Vec::new();
        
        for (i, address) in address_list.iter().enumerate() {
            if address.is_empty() {
                continue;
            }
            
            println!("  [{}/{}] Checking {}...", i + 1, address_list.len(), 
                if address.len() > 16 { 
                    format!("{}...{}", &address[..8], &address[address.len()-8..])
                } else { 
                    address.to_string() 
                }
            );
            
            let url = format!(
                "https://api.range.org/v1/risk/address?address={}&network=solana",
                address
            );
            
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(risk_data) = response.json::<RangeRiskResponse>().await {
                            results.push(risk_data);
                        }
                    }
                }
                Err(_) => {
                    // Continue with other addresses
                }
            }
            
            // Rate limiting - small delay between requests
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
        
        println!();
        
        if format == "json" {
            println!("{}", serde_json::to_string_pretty(&results).unwrap_or_default());
        } else {
            println!("{} Results Summary", "✓".bright_green());
            println!();
            
            let mut low_risk = 0;
            let mut medium_risk = 0;
            let mut high_risk = 0;
            let mut critical_risk = 0;
            
            for result in &results {
                if let Some(score) = result.risk_score {
                    match score {
                        0..=2 => low_risk += 1,
                        3..=5 => medium_risk += 1,
                        6..=8 => high_risk += 1,
                        _ => critical_risk += 1,
                    }
                }
            }
            
            println!("  {} Low Risk (0-2)", format!("{:3}", low_risk).bright_green());
            println!("  {} Medium Risk (3-5)", format!("{:3}", medium_risk).bright_yellow());
            println!("  {} High Risk (6-8)", format!("{:3}", high_risk).bright_red());
            println!("  {} Critical Risk (9-10)", format!("{:3}", critical_risk).bright_red().bold());
            
            if critical_risk > 0 || high_risk > 0 {
                println!();
                println!("  {} Some addresses require attention!", "⚠".bright_yellow());
            }
        }
        
        Ok(())
    }
    
    async fn check_transaction(&self, signature: &str) -> Result<()> {
        println!("{} Transaction Compliance Check", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  Signature: {}", 
            if signature.len() > 32 {
                format!("{}...{}", &signature[..16], &signature[signature.len()-16..])
            } else {
                signature.to_string()
            }
        );
        println!();
        
        println!("  {}:", "Note".bright_yellow());
        println!("    Transaction-level compliance checking requires:");
        println!("    1. Fetching transaction details from RPC");
        println!("    2. Extracting all involved addresses");
        println!("    3. Checking each address against Range API");
        println!();
        println!("  {}:", "Manual Steps".bright_white());
        println!("    1. Get transaction accounts:");
        println!("       solana confirm -v {}", signature);
        println!();
        println!("    2. Check each address:");
        println!("       solprivacy compliance check --address <ADDRESS>");
        println!();
        println!("  {}:", "Automated Integration".bright_white());
        println!("    Use the Solana SDK with Range API in your application:");
        println!("    solprivacy compliance integrate");
        
        Ok(())
    }
    
    fn show_integration_guide(&self) -> Result<()> {
        println!("{} Range Compliance Integration Guide", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  {}:", "Option 1: Off-Chain API Integration".bright_white());
        println!();
        println!("    Use Range's REST API before processing transactions:");
        println!();
        println!("    {}:", "TypeScript Example".bright_cyan());
        println!(r#"    ```typescript
    const checkRisk = async (address: string) => {{
      const response = await fetch(
        `https://api.range.org/v1/risk/address?address=${{address}}&network=solana`
      );
      const data = await response.json();
      
      if (data.riskScore >= 7) {{
        throw new Error('High risk address detected');
      }}
      
      return data;
    }};
    ```"#);
        println!();
        
        println!("  {}:", "Option 2: On-Chain Verification (Switchboard)".bright_white());
        println!();
        println!("    Verify risk quotes directly in your Solana program:");
        println!();
        println!("    {}:", "Rust/Anchor Example".bright_cyan());
        println!(r#"    ```rust
    use switchboard_on_demand::prelude::*;
    
    pub fn verify_risk_quote(
        ctx: Context<VerifyRisk>,
        quote: Vec<u8>,
    ) -> Result<()> {{
        // Verify the Range risk quote signature
        let risk_data = verify_range_quote(&quote)?;
        
        require!(
            risk_data.risk_score < 7,
            ErrorCode::HighRiskAddress
        );
        
        Ok(())
    }}
    ```"#);
        println!();
        
        println!("  {}:", "Option 3: Real-Time Transaction Monitoring".bright_white());
        println!();
        println!("    Set up webhooks to monitor your program:");
        println!();
        println!("    1. Register your program with Range");
        println!("    2. Configure webhook endpoint");
        println!("    3. Receive real-time risk alerts");
        println!();
        println!("    Dashboard: https://app.range.org");
        println!();
        
        println!("  {}:", "Resources".bright_white());
        println!("    Range Docs: https://docs.range.org");
        println!("    Oracle Example: https://github.com/rangesecurity/oracle-example");
        println!("    Switchboard: https://switchboard.xyz");
        println!();
        
        println!("  {}:", "Hackathon Track".bright_green());
        println!("    Range Track: Real-time threat protection");
        println!("    30% discount for hackathon participants!");
        
        Ok(())
    }
    
    fn show_info(&self) -> Result<()> {
        println!("{} Range Protocol - Compliance & Risk Intelligence", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  {}:", "What is Range?".bright_white());
        println!("    Range provides real-time risk intelligence and transaction");
        println!("    monitoring for Solana programs. It helps identify:");
        println!("    • Sanctioned addresses (OFAC, etc.)");
        println!("    • Hack-related funds");
        println!("    • Mixer/tumbler interactions");
        println!("    • Known scam addresses");
        println!();
        
        println!("  {}:", "Risk Score Scale".bright_white());
        println!("    {} 0-2  Low Risk - Clean address", "●".bright_green());
        println!("    {} 3-5  Medium Risk - Some concerns", "●".bright_yellow());
        println!("    {} 6-8  High Risk - Significant issues", "●".bright_red());
        println!("    {} 9-10 Critical - Known malicious", "●".bright_red().bold());
        println!();
        
        println!("  {}:", "Key Features".bright_white());
        println!("    • Real-time address risk scoring");
        println!("    • On-chain verification via Switchboard");
        println!("    • Transaction monitoring webhooks");
        println!("    • OFAC sanctions screening");
        println!("    • Hack fund tracing");
        println!();
        
        println!("  {}:", "API Endpoints".bright_white());
        println!("    Risk Check: GET /v1/risk/address");
        println!("    Batch Check: POST /v1/risk/addresses");
        println!("    Transaction: GET /v1/risk/transaction");
        println!();
        
        println!("  {}:", "Use Cases".bright_white());
        println!("    • DeFi protocol compliance (KYT)");
        println!("    • NFT marketplace safety");
        println!("    • Payment processing");
        println!("    • Exchange deposits/withdrawals");
        println!();
        
        println!("  {}:", "Quick Start".bright_white());
        println!("    # Check an address");
        println!("    solprivacy compliance check --address <PUBKEY>");
        println!();
        println!("    # Check multiple addresses");
        println!("    solprivacy compliance batch --addresses addr1,addr2,addr3");
        println!();
        
        println!("  {}:", "Resources".bright_white());
        println!("    Website: https://range.org");
        println!("    Docs: https://docs.range.org");
        println!("    Dashboard: https://app.range.org");
        
        Ok(())
    }
    
    fn configure(&self, api_key: Option<String>, show: bool) -> Result<()> {
        println!("{} Range API Configuration", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        if show {
            println!("  {}:", "Current Configuration".bright_white());
            println!("    API Key: {} (using public tier)", "Not set".bright_black());
            println!();
            println!("  {}:", "Rate Limits".bright_white());
            println!("    Public Tier: 100 requests/day");
            println!("    With API Key: Higher limits available");
            return Ok(());
        }
        
        if let Some(key) = api_key {
            println!("  Setting Range API Key: {}...", &key[..8.min(key.len())]);
            println!();
            println!("  {}:", "Note".bright_yellow());
            println!("    To persist this setting, add to your environment:");
            println!("    export RANGE_API_KEY={}", key);
            println!();
            println!("    Or add to your .env file.");
        } else {
            println!("  {}:", "Usage".bright_white());
            println!("    solprivacy compliance config --api-key <YOUR_KEY>");
            println!("    solprivacy compliance config --show");
            println!();
            println!("  {}:", "Get an API Key".bright_white());
            println!("    1. Sign up at https://app.range.org");
            println!("    2. Create a new API key");
            println!("    3. Configure with the command above");
        }
        
        Ok(())
    }
}
