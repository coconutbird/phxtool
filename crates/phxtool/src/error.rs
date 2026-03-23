//! Unified error type for phxtool operations.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ERA error: {0}")]
    Era(#[from] era::Error),

    #[error("XMB error: {0}")]
    Xmb(#[from] xmb::Error),

    #[error("ECF error: {0}")]
    Ecf(#[from] ecf::Error),

    #[error("file not found: {0}")]
    FileNotFound(PathBuf),

    #[error("invalid archive format: {0}")]
    InvalidFormat(String),

    #[error("operation cancelled")]
    Cancelled,

    #[error("Wwise error: {0}")]
    Wwise(#[from] pcktool::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
