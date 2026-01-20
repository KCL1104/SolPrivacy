use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::error::{Result, SolPrivacyError};

/// Application configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    /// Current network (devnet, mainnet, localnet)
    #[serde(default = "default_network")]
    pub network: String,
    
    /// RPC configuration
    #[serde(default)]
    pub rpc: RpcConfig,
}

fn default_network() -> String {
    "devnet".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RpcConfig {
    /// Helius API key
    pub helius_api_key: Option<String>,
    
    /// QuickNode endpoint
    pub quicknode_endpoint: Option<String>,
    
    /// Custom RPC URL
    pub custom_rpc_url: Option<String>,
    
    /// Active provider: "helius", "quicknode", "custom", or "default"
    #[serde(default = "default_provider")]
    pub active_provider: String,
}

fn default_provider() -> String {
    "default".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: default_network(),
            rpc: RpcConfig::default(),
        }
    }
}

impl AppConfig {
    /// Get the config directory path
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("solprivacy")
    }
    
    /// Get the config file path
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }
    
    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| SolPrivacyError::Config(format!("Failed to read config: {}", e)))?;
            let config: AppConfig = serde_json::from_str(&content)
                .map_err(|e| SolPrivacyError::Config(format!("Failed to parse config: {}", e)))?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| SolPrivacyError::Config(format!("Failed to create config dir: {}", e)))?;
        
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| SolPrivacyError::Config(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&path, content)
            .map_err(|e| SolPrivacyError::Config(format!("Failed to write config: {}", e)))?;
        
        Ok(())
    }
    
    /// Get the active RPC URL based on configuration
    pub fn get_rpc_url(&self) -> String {
        match self.rpc.active_provider.as_str() {
            "helius" => {
                if let Some(ref key) = self.rpc.helius_api_key {
                    match self.network.as_str() {
                        "mainnet" => format!("https://mainnet.helius-rpc.com/?api-key={}", key),
                        _ => format!("https://devnet.helius-rpc.com/?api-key={}", key),
                    }
                } else {
                    self.default_rpc_url()
                }
            }
            "quicknode" => {
                self.rpc.quicknode_endpoint.clone().unwrap_or_else(|| self.default_rpc_url())
            }
            "custom" => {
                self.rpc.custom_rpc_url.clone().unwrap_or_else(|| self.default_rpc_url())
            }
            _ => self.default_rpc_url(),
        }
    }
    
    fn default_rpc_url(&self) -> String {
        match self.network.as_str() {
            "mainnet" => "https://api.mainnet-beta.solana.com".to_string(),
            "localnet" => "http://127.0.0.1:8899".to_string(),
            _ => "https://api.devnet.solana.com".to_string(),
        }
    }
}
