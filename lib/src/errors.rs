use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("File processing error: {0}")]
    FileProcessing(String),

    #[error("WeChat cache not found")]
    CacheNotFound,

    #[error("UI rendering error: {0}")]
    UiRendering(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
