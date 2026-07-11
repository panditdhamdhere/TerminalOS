//! Workspace state and session restoration.

pub mod id;
pub mod manager;
pub mod snapshot;
pub mod store;

pub use id::id_from_path;
pub use manager::{Workspace, WorkspaceManager, tabs_from_session};
pub use snapshot::{TabSnapshot, UiSnapshot, WorkspaceSnapshot, WorkspaceSummary};
pub use store::WorkspaceStore;
