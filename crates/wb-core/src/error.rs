use thiserror::Error;

#[derive(Error, Debug)]
pub enum WbError {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Collector error: {0}")]
    Collector(String),
    #[error("AI error: {0}")]
    Ai(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, WbError>;
