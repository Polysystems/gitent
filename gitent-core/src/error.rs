use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Change not found: {0}")]
    ChangeNotFound(String),

    #[error("Commit not found: {0}")]
    CommitNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Diff generation failed: {0}")]
    DiffFailed(String),

    #[error("No active session")]
    NoActiveSession,

    #[error("Session already active at: {0}")]
    SessionAlreadyActive(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
