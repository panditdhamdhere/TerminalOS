//! TerminalOS terminal user interface built with Ratatui.

pub mod app;
pub mod components;
pub mod event;
pub mod layout;
pub mod theme;

pub use app::{TerminalApp, TerminalAppOptions};
