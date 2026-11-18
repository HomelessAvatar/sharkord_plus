use thiserror::Error;

#[derive(Error, Debug)]
pub enum SharkordError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("tRPC error ({code}): {message}")]
    Trpc { code: i32, message: String },

    #[error("RPC error ({code}): {message}")]
    RpcError { code: i32, message: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for SharkordError {
    fn from(err: reqwest::Error) -> Self {
        SharkordError::Network(err.to_string())
    }
}

impl From<url::ParseError> for SharkordError {
    fn from(err: url::ParseError) -> Self {
        SharkordError::Protocol(format!("Invalid URL: {err}"))
    }
}
