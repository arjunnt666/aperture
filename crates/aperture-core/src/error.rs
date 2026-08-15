use thiserror::Error;

pub type Result<T> = std::result::Result<T, ApertureError>;

#[derive(Debug, Error)]
pub enum ApertureError {
    #[error("limit exceeded")] LimitExceeded,
    #[error("circuit open")] CircuitOpen,
    #[error("bulkhead full")] BulkheadFull,
    #[error("invalid config: {0}")] InvalidConfig(String),
    #[error("internal: {0}")] Internal(String),
}
