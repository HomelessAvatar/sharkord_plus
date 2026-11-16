use thiserror::Error;

#[derive(Error, Debug)]
pub enum SharkordError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("tRPC error ({code}): {message}")]
    Trpc { code: i32, message: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}
