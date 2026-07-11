use serde::{Deserialize, Serialize};
use terminalos_shared::{SessionId, TabId, WorkspaceId};
use uuid::Uuid;

/// Top-level protocol message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub kind: MessageKind,
}

impl Message {
    #[must_use]
    pub fn new(kind: MessageKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind,
        }
    }
}

/// Variants of messages exchanged between CLI, daemon, and terminal app.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum MessageKind {
    DaemonRequest(DaemonRequest),
    DaemonResponse(DaemonResponse),
    Ping,
    Pong,
}

/// Requests sent to the background daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "data")]
pub enum DaemonRequest {
    HealthCheck,
    ListWorkspaces,
    OpenWorkspace {
        path: String,
    },
    CloseWorkspace {
        id: WorkspaceId,
    },
    CreateSession {
        workspace_id: WorkspaceId,
    },
    CloseSession {
        id: SessionId,
    },
    CreateTab {
        session_id: SessionId,
        title: String,
    },
    CloseTab {
        id: TabId,
    },
}

/// Responses from the background daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum DaemonResponse {
    Ok { message: String },
    Error { code: u16, message: String },
    WorkspaceList { workspaces: Vec<WorkspaceInfo> },
}

/// Summary information about a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: WorkspaceId,
    pub path: String,
    pub name: String,
    pub branch: Option<String>,
}
