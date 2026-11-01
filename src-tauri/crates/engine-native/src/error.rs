use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid source file {path}: {message}")]
    InvalidSourceFile { path: PathBuf, message: String },

    #[error("invalid source: {0}")]
    InvalidSource(String),

    #[error("unsupported feature: {0}")]
    Unsupported(String),

    #[error("rule error: {0}")]
    Rule(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
