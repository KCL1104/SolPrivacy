use clap::{Args, Subcommand};
use colored::Colorize;
use solana_client::rpc_client::RpcClient;
use crate::config::AppConfig;
use crate::error::Result;

/// Configure RPC providers and environment
#[derive(Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Configure Helius RPC
    Helius {
        /// Helius API key
        #[arg(long, env = "HELIUS_API_KEY")]
        api_key: String,
    },
    
    /// Configure QuickNode RPC
    Quicknode {
        /// QuickNode endpoint URL
        #[arg(long)]
        endpoint: String,
    },

    /// Configure custom RPC URL
    Custom {
        /// Custom RPC URL
        #[arg(long)]
        url: String,
    },
    
    /// Set network (devnet/mainnet)
    Network {
        /// Network to use
        #[arg(value_parser = ["devnet", "mainnet", "localnet"])]
        network: String,
    },
    
    /// Show current configuration
    Show,
    
    /// Show the active RPC URL
    Rpc,

    /// Test RPC connectivity
    Test {
        /// Optional RPC URL to test (defaults to current config)
        #[arg(long)]
        url: Option<String>,
    },
}

impl ConfigCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            ConfigAction::Helius { api_key } => {
                println!("{} Configuring Helius RPC...", "→".bright_cyan());
                
                let mut config = AppConfig::load()?;
                config.rpc.helius_api_key = Some(api_key.clone());
                config.rpc.active_provider = "helius".to_string();
                config.save()?;
                
                println!("{} Helius configuration saved!", "✓".bright_green());
                println!("  API Key: {}...", &api_key[..8.min(api_key.len())]);
                println!("  Provider set to: {}", "helius".bright_white());
                println!();
                println!("  RPC URL: {}", config.get_rpc_url().bright_blue());
            }
            ConfigAction::Quicknode { endpoint } => {
                println!("{} Configuring QuickNode RPC...", "→".bright_cyan());
                
                let mut config = AppConfig::load()?;
                config.rpc.quicknode_endpoint = Some(endpoint.clone());
                config.rpc.active_provider = "quicknode".to_string();
                config.save()?;
                
                println!("{} QuickNode configuration saved!", "✓".bright_green());
                println!("  Endpoint: {}", endpoint);
                println!("  Provider set to: {}", "quicknode".bright_white());
            }
            ConfigAction::Custom { url } => {
                println!("{} Configuring custom RPC...", "→".bright_cyan());

                let mut config = AppConfig::load()?;
                config.rpc.custom_rpc_url = Some(url.clone());
                config.rpc.active_provider = "custom".to_string();
                config.save()?;

                println!("{} Custom RPC configuration saved!", "✓".bright_green());
                println!("  URL: {}", url);
                println!("  Provider set to: {}", "custom".bright_white());
            }
            ConfigAction::Network { network } => {
                println!("{} Setting network...", "→".bright_cyan());
                
                let mut config = AppConfig::load()?;
                config.network = network.clone();
                config.save()?;
                
                println!("{} Network set to: {}", "✓".bright_green(), network.bright_white());
                println!("  RPC URL: {}", config.get_rpc_url().bright_blue());
            }
            ConfigAction::Show => {
                let config = AppConfig::load()?;
                
                println!("{} Current Configuration:", "ℹ".bright_blue());
                println!();
                println!("  {}:", "Network".bright_white());
                println!("    {}", config.network);
                println!();
                println!("  {}:", "RPC Provider".bright_white());
                println!("    Active: {}", config.rpc.active_provider.bright_cyan());
                
                if let Some(ref key) = config.rpc.helius_api_key {
                    println!("    Helius API Key: {}...", &key[..8.min(key.len())]);
                }
                if let Some(ref endpoint) = config.rpc.quicknode_endpoint {
                    println!("    QuickNode Endpoint: {}", endpoint);
                }
                if let Some(ref url) = config.rpc.custom_rpc_url {
                    println!("    Custom RPC URL: {}", url);
                }
                
                println!();
                println!("  {}:", "Active RPC URL".bright_white());
                println!("    {}", config.get_rpc_url().bright_blue());
                println!();
                println!("  {}:", "Config File".bright_white());
                println!("    {}", AppConfig::config_path().display());
            }
            ConfigAction::Rpc => {
                let config = AppConfig::load()?;
                // Just print the RPC URL for scripting use
                println!("{}", config.get_rpc_url());
            }
            ConfigAction::Test { url } => {
                let config = AppConfig::load()?;
                let target_url = url.clone().unwrap_or_else(|| config.get_rpc_url());

                println!("{} Testing RPC...", "→".bright_cyan());
                println!("  URL: {}", target_url.bright_blue());

                let client = RpcClient::new(target_url.clone());
                match client.get_version() {
                    Ok(version) => {
                        let feature_set = version
                            .feature_set
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        println!("{} RPC reachable", "✓".bright_green());
                        println!("  Solana core: {}", version.solana_core);
                        println!("  Feature set: {}", feature_set);
                    }
                    Err(error) => {
                        println!("{} RPC unreachable", "✗".bright_red());
                        println!("  Error: {}", error);
                    }
                }
            }
        }
        
        Ok(())
    }
}
