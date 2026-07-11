use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use terminalos_shared::{TabId, WorkspaceId};

/// Persisted terminal tab state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TabSnapshot {
    pub id: TabId,
    pub title: String,
    pub cwd: String,
    pub position: usize,
}

/// Persisted UI pane visibility and focus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiSnapshot {
    pub show_sidebar: bool,
    pub show_chat: bool,
    pub show_logs: bool,
    pub focus_pane: String,
    pub active_tab: usize,
}

impl Default for UiSnapshot {
    fn default() -> Self {
        Self {
            show_sidebar: true,
            show_chat: true,
            show_logs: true,
            focus_pane: "terminal".to_string(),
            active_tab: 0,
        }
    }
}

/// Full workspace session snapshot for restoration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub workspace_id: WorkspaceId,
    pub path: String,
    pub name: String,
    pub branch: Option<String>,
    pub tabs: Vec<TabSnapshot>,
    pub env: HashMap<String, String>,
    pub ui: UiSnapshot,
    pub saved_at: DateTime<Utc>,
}

/// Summary of a recently opened workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub id: WorkspaceId,
    pub path: String,
    pub name: String,
    pub branch: Option<String>,
    pub last_opened_at: DateTime<Utc>,
}
