use serde::{Deserialize, Serialize};

/// Kind of action awaiting user confirmation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    WriteFile,
    CreateFile,
    DeleteFile,
    RenameFile,
    RunCommand,
}

/// A destructive or shell action waiting for explicit user approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    pub kind: ActionKind,
    pub summary: String,
    pub preview: String,
    pub path: Option<String>,
    pub destination: Option<String>,
    pub content: Option<String>,
    pub command: Option<String>,
}

impl PendingAction {
    #[must_use]
    pub fn write_file(path: String, preview: String, content: String) -> Self {
        Self {
            kind: ActionKind::WriteFile,
            summary: format!("Write changes to {path}"),
            preview,
            path: Some(path),
            destination: None,
            content: Some(content),
            command: None,
        }
    }

    #[must_use]
    pub fn create_file(path: String, preview: String, content: String) -> Self {
        Self {
            kind: ActionKind::CreateFile,
            summary: format!("Create file {path}"),
            preview,
            path: Some(path),
            destination: None,
            content: Some(content),
            command: None,
        }
    }

    #[must_use]
    pub fn delete_file(path: String) -> Self {
        Self {
            kind: ActionKind::DeleteFile,
            summary: format!("Delete {path}"),
            preview: String::new(),
            path: Some(path),
            destination: None,
            content: None,
            command: None,
        }
    }

    #[must_use]
    pub fn rename_file(from: String, to: String) -> Self {
        Self {
            kind: ActionKind::RenameFile,
            summary: format!("Rename {from} → {to}"),
            preview: String::new(),
            path: Some(from),
            destination: Some(to),
            content: None,
            command: None,
        }
    }

    #[must_use]
    pub fn run_command(command: String) -> Self {
        Self {
            kind: ActionKind::RunCommand,
            summary: format!("Run: {command}"),
            preview: command.clone(),
            path: None,
            destination: None,
            content: None,
            command: Some(command),
        }
    }
}
