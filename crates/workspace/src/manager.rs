use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use terminalos_git::{GitRepository, RepoStatus};
use terminalos_shared::{Result, WorkspaceId};

/// A developer workspace rooted at a project directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub path: PathBuf,
    pub name: String,
    pub branch: Option<String>,
    pub env: HashMap<String, String>,
}

/// Manages open workspaces and their metadata.
#[derive(Debug, Default)]
pub struct WorkspaceManager {
    workspaces: HashMap<WorkspaceId, Workspace>,
    active: Option<WorkspaceId>,
}

impl WorkspaceManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<WorkspaceId> {
        let path = path
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| path.as_ref().to_path_buf());
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());

        let branch = GitRepository::discover(&path)
            .ok()
            .and_then(|repo| repo.status().ok())
            .and_then(|s| s.branch);

        let id = WorkspaceId::new();
        let workspace = Workspace {
            id,
            path,
            name,
            branch,
            env: HashMap::new(),
        };

        self.workspaces.insert(id, workspace);
        self.active = Some(id);
        Ok(id)
    }

    #[must_use]
    pub fn active(&self) -> Option<&Workspace> {
        self.active.and_then(|id| self.workspaces.get(&id))
    }

    #[must_use]
    pub fn get(&self, id: WorkspaceId) -> Option<&Workspace> {
        self.workspaces.get(&id)
    }

    pub fn refresh_git_status(&mut self, id: WorkspaceId) -> Result<RepoStatus> {
        let workspace = self.workspaces.get(&id).ok_or_else(|| {
            terminalos_shared::Error::Workspace("workspace not found".to_string())
        })?;

        let repo = GitRepository::discover(&workspace.path)?;
        let status = repo.status()?;

        if let Some(ws) = self.workspaces.get_mut(&id) {
            ws.branch = status.branch.clone();
        }

        Ok(status)
    }

    #[must_use]
    pub fn list(&self) -> Vec<&Workspace> {
        self.workspaces.values().collect()
    }
}
