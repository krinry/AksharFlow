use thiserror::Error;

/// Domain errors returned by the AksharFlow core typing engine.
#[derive(Debug, Error)]
pub enum AksharError {
    #[error("No active typing session")]
    SessionNotActive,
    #[error("Unsupported physical key code: {0}")]
    UnsupportedKeyCode(String),
    #[error("Corpus for layout {0} has no available words")]
    EmptyCorpus(String),
    #[error("Failed to parse JSON payload: {0}")]
    CorpusParse(#[from] serde_json::Error),
    #[cfg(feature = "sqlite")]
    #[error("SQLite storage error: {0}")]
    Storage(#[from] rusqlite::Error),
    #[error("Storage error: {0}")]
    StorageMessage(String),
    #[error("Invalid backup bundle: {0}")]
    InvalidBackup(String),
    #[error("Filesystem I/O error: {0}")]
    Io(#[from] std::io::Error),
}
