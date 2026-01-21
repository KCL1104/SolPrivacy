use thiserror::Error;

pub type Result<T> = std::result::Result<T, SolPrivacyError>;

#[derive(Error, Debug)]
pub enum SolPrivacyError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("{0}")]
    Other(String),
}
