//! Message protocol for communication between TerminalOS components.

pub mod messages;

pub use messages::{DaemonRequest, DaemonResponse, Message, MessageKind};
