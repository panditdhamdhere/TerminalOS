use thiserror::Error;

/// Unified error type for TerminalOS.
#[derive(Debug, Error)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("filesystem error: {0}")]
    Filesystem(String),

    #[error("git error: {0}")]
    Git(String),

    #[error("ai provider error: {0}")]
    Ai(String),

    #[error("terminal error: {0}")]
    Terminal(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("plugin error: {0}")]
    Plugin(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("search error: {0}")]
    Search(String),

    #[error("workspace error: {0}")]
    Workspace(String),

    #[error("ui error: {0}")]
    Ui(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
