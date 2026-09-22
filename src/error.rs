use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Input error: {0}")]
    InvalidInput(String),

    #[error("Dictionary entry not found: {0}")]
    NotFound(String),

    #[error("Remote response error: {0}")]
    Protocol(String),

    #[error("{0}")]
    General(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
