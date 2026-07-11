use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use terminalos_git::{GitRepository, RepoStatus};
use terminalos_shared::{Result, TabId, WorkspaceId};

use crate::id::id_from_path;
use crate::snapshot::{TabSnapshot, UiSnapshot, WorkspaceSnapshot};

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
        let id = id_from_path(&path);

        if self.workspaces.contains_key(&id) {
            self.active = Some(id);
            if let Some(ws) = self.workspaces.get_mut(&id) {
                ws.branch = GitRepository::discover(&path)
                    .ok()
                    .and_then(|repo| repo.status().ok())
                    .and_then(|s| s.branch);
            }
            return Ok(id);
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());

        let branch = GitRepository::discover(&path)
            .ok()
            .and_then(|repo| repo.status().ok())
            .and_then(|s| s.branch);

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

    pub fn apply_snapshot(&mut self, snapshot: &WorkspaceSnapshot) -> Result<WorkspaceId> {
        let path = PathBuf::from(&snapshot.path);
        let id = self.open(&path)?;
        if let Some(ws) = self.workspaces.get_mut(&id) {
            ws.branch = snapshot.branch.clone();
            ws.env = snapshot.env.clone();
        }
        Ok(id)
    }

    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        let id = self.active_workspace_id()?;
        let key = key.into();
        let value = value.into();
        if let Some(ws) = self.workspaces.get_mut(&id) {
            ws.env.insert(key, value);
        }
        Ok(())
    }

    pub fn remove_env(&mut self, key: &str) -> Result<()> {
        let id = self.active_workspace_id()?;
        if let Some(ws) = self.workspaces.get_mut(&id) {
            ws.env.remove(key);
        }
        Ok(())
    }

    #[must_use]
    pub fn env(&self) -> HashMap<String, String> {
        self.active().map(|ws| ws.env.clone()).unwrap_or_default()
    }

    pub fn build_snapshot(
        &self,
        tabs: &[TabSnapshot],
        ui: UiSnapshot,
    ) -> Result<WorkspaceSnapshot> {
        let ws = self.active().ok_or_else(|| {
            terminalos_shared::Error::Workspace("no active workspace".to_string())
        })?;
        Ok(WorkspaceSnapshot {
            workspace_id: ws.id,
            path: ws.path.display().to_string(),
            name: ws.name.clone(),
            branch: ws.branch.clone(),
            tabs: tabs.to_vec(),
            env: ws.env.clone(),
            ui,
            saved_at: Utc::now(),
        })
    }

    #[must_use]
    pub fn active(&self) -> Option<&Workspace> {
        self.active.and_then(|id| self.workspaces.get(&id))
    }

    #[must_use]
    pub fn active_id(&self) -> Option<WorkspaceId> {
        self.active
    }

    fn active_workspace_id(&self) -> Result<WorkspaceId> {
        self.active
            .ok_or_else(|| terminalos_shared::Error::Workspace("no active workspace".to_string()))
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

/// Builds tab snapshots from a shell session.
#[must_use]
pub fn tabs_from_session(tabs: &[(TabId, String, String)]) -> Vec<TabSnapshot> {
    tabs.iter()
        .enumerate()
        .map(|(position, (id, title, cwd))| TabSnapshot {
            id: *id,
            title: title.clone(),
            cwd: cwd.clone(),
            position,
        })
        .collect()
}
