use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severity level for application log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// A single application log entry displayed in the bottom log pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    #[must_use]
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Info, message)
    }

    #[must_use]
    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Warn, message)
    }

    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(LogLevel::Error, message)
    }
}
