use thiserror::Error;

pub type Result<T> = std::result::Result<T, SolPrivacyError>;

#[derive(Error, Debug)]
pub enum SolPrivacyError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("RPC error: {0}")]
    Rpc(String),
    
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("Solana client error: {0}")]
    SolanaClient(String),
    
    #[error("{0}")]
    Other(String),
}
