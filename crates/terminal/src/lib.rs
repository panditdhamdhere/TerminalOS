//! Terminal emulator with PTY-backed shell sessions.

pub mod emulator;
pub mod history;
pub mod keys;
pub mod layout;
pub mod manager;
pub mod pty;
pub mod session;

pub use emulator::{StyledSpan, TerminalEmulator};
pub use history::CommandHistory;
pub use keys::{is_scroll_key, key_event_to_bytes};
pub use layout::{
    Area, SplitDirection, SplitNode, compute_pane_rects, deserialize_layout, serialize_layout,
};
pub use manager::ShellManager;
pub use pty::{PtyOutput, PtySession};
pub use session::{ShellSession, TerminalTab};
