use thiserror::Error;
use solana_pubkey::ParsePubkeyError;

pub type Result<T> = std::result::Result<T, SolPrivacyError>;

#[derive(Error, Debug)]
pub enum SolPrivacyError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Cryptography error: {0}")]
    Crypto(String),
    
    #[error("RPC Client error: {0}")]
    Rpc(String),

    #[error("Solana Client error: {0}")]
    SolanaClient(String),

    #[error("Invalid Public Key: {0}")]
    InvalidPubkey(#[from] ParsePubkeyError),

    #[error("External Tool Missing: {0}. Please install it to proceed.")]
    ToolMissing(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Input error: {0}")]
    Input(String),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Unknown error: {0}")]
    Other(String),
}
