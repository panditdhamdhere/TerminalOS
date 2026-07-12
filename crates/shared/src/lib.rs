//! Shared types, errors, and utilities used across TerminalOS crates.

pub mod error;
pub mod ids;
pub mod log;
pub mod theme;

pub use error::{Error, Result};
pub use ids::{PaneId, SessionId, TabId, WorkspaceId};
pub use log::{LogEntry, LogLevel};
pub use theme::{Theme, ThemeMode};
