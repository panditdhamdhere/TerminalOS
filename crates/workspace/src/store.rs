use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use terminalos_shared::{Error, Result, TabId, WorkspaceId};
use uuid::Uuid;

use crate::snapshot::{TabSnapshot, UiSnapshot, WorkspaceSnapshot, WorkspaceSummary};

/// SQLite persistence for workspace sessions.
pub struct WorkspaceStore {
    pool: SqlitePool,
}

impl WorkspaceStore {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Database(format!("create dir failed: {e}")))?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.display()))
            .map_err(|e| Error::Database(format!("invalid sqlite options: {e}")))?
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .map_err(|e| Error::Database(format!("connect failed: {e}")))?;

        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY NOT NULL,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                branch TEXT,
                last_opened_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS workspace_env (
                workspace_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY (workspace_id, key),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
            );
            CREATE TABLE IF NOT EXISTS terminal_tabs (
                workspace_id TEXT NOT NULL,
                tab_id TEXT NOT NULL,
                title TEXT NOT NULL,
                cwd TEXT NOT NULL,
                position INTEGER NOT NULL,
                PRIMARY KEY (workspace_id, tab_id),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
            );
            CREATE TABLE IF NOT EXISTS workspace_ui (
                workspace_id TEXT PRIMARY KEY NOT NULL,
                active_tab INTEGER NOT NULL DEFAULT 0,
                show_sidebar INTEGER NOT NULL DEFAULT 1,
                show_chat INTEGER NOT NULL DEFAULT 1,
                show_logs INTEGER NOT NULL DEFAULT 1,
                focus_pane TEXT NOT NULL DEFAULT 'terminal',
                FOREIGN KEY(workspace_id) REFERENCES workspaces(id)
            );
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("migration failed: {e}")))?;

        Ok(())
    }

    pub async fn save_snapshot(&self, snapshot: &WorkspaceSnapshot) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Database(format!("begin tx: {e}")))?;

        sqlx::query(
            "INSERT OR REPLACE INTO workspaces (id, path, name, branch, last_opened_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(snapshot.workspace_id.as_uuid().to_string())
        .bind(&snapshot.path)
        .bind(&snapshot.name)
        .bind(&snapshot.branch)
        .bind(snapshot.saved_at.to_rfc3339())
        .execute(&mut *tx)
        .await
        .map_err(|e| Error::Database(format!("save workspace: {e}")))?;

        let ws_id = snapshot.workspace_id.as_uuid().to_string();
        sqlx::query("DELETE FROM workspace_env WHERE workspace_id = ?")
            .bind(&ws_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(format!("clear env: {e}")))?;

        for (key, value) in &snapshot.env {
            sqlx::query("INSERT INTO workspace_env (workspace_id, key, value) VALUES (?, ?, ?)")
                .bind(&ws_id)
                .bind(key)
                .bind(value)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Database(format!("save env: {e}")))?;
        }

        sqlx::query("DELETE FROM terminal_tabs WHERE workspace_id = ?")
            .bind(&ws_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(format!("clear tabs: {e}")))?;

        for tab in &snapshot.tabs {
            sqlx::query(
                "INSERT INTO terminal_tabs (workspace_id, tab_id, title, cwd, position) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&ws_id)
            .bind(tab.id.as_uuid().to_string())
            .bind(&tab.title)
            .bind(&tab.cwd)
            .bind(tab.position as i64)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(format!("save tab: {e}")))?;
        }

        sqlx::query(
            "INSERT OR REPLACE INTO workspace_ui (workspace_id, active_tab, show_sidebar, show_chat, show_logs, focus_pane) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&ws_id)
        .bind(snapshot.ui.active_tab as i64)
        .bind(i64::from(snapshot.ui.show_sidebar))
        .bind(i64::from(snapshot.ui.show_chat))
        .bind(i64::from(snapshot.ui.show_logs))
        .bind(&snapshot.ui.focus_pane)
        .execute(&mut *tx)
        .await
        .map_err(|e| Error::Database(format!("save ui: {e}")))?;

        tx.commit()
            .await
            .map_err(|e| Error::Database(format!("commit: {e}")))?;
        Ok(())
    }

    pub async fn load_snapshot(
        &self,
        workspace_id: WorkspaceId,
    ) -> Result<Option<WorkspaceSnapshot>> {
        let ws_id = workspace_id.as_uuid().to_string();
        let row = sqlx::query(
            "SELECT id, path, name, branch, last_opened_at FROM workspaces WHERE id = ?",
        )
        .bind(&ws_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("load workspace: {e}")))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let path: String = row.get("path");
        let name: String = row.get("name");
        let branch: Option<String> = row.get("branch");
        let last_opened: String = row.get("last_opened_at");
        let saved_at = DateTime::parse_from_rfc3339(&last_opened)
            .map_err(|e| Error::Database(format!("invalid timestamp: {e}")))?
            .with_timezone(&Utc);

        let env_rows = sqlx::query("SELECT key, value FROM workspace_env WHERE workspace_id = ?")
            .bind(&ws_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("load env: {e}")))?;

        let mut env = HashMap::new();
        for env_row in env_rows {
            let key: String = env_row.get("key");
            let value: String = env_row.get("value");
            env.insert(key, value);
        }

        let tab_rows = sqlx::query(
            "SELECT tab_id, title, cwd, position FROM terminal_tabs WHERE workspace_id = ? ORDER BY position ASC",
        )
        .bind(&ws_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("load tabs: {e}")))?;

        let mut tabs = Vec::with_capacity(tab_rows.len());
        for tab_row in tab_rows {
            let tab_id: String = tab_row.get("tab_id");
            let title: String = tab_row.get("title");
            let cwd: String = tab_row.get("cwd");
            let position: i64 = tab_row.get("position");
            tabs.push(TabSnapshot {
                id: TabId::from_uuid(
                    Uuid::parse_str(&tab_id)
                        .map_err(|e| Error::Database(format!("invalid tab uuid: {e}")))?,
                ),
                title,
                cwd,
                position: position as usize,
            });
        }

        let ui_row = sqlx::query(
            "SELECT active_tab, show_sidebar, show_chat, show_logs, focus_pane FROM workspace_ui WHERE workspace_id = ?",
        )
        .bind(&ws_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("load ui: {e}")))?;

        let ui = if let Some(ui_row) = ui_row {
            UiSnapshot {
                active_tab: ui_row.get::<i64, _>("active_tab") as usize,
                show_sidebar: ui_row.get::<i64, _>("show_sidebar") != 0,
                show_chat: ui_row.get::<i64, _>("show_chat") != 0,
                show_logs: ui_row.get::<i64, _>("show_logs") != 0,
                focus_pane: ui_row.get("focus_pane"),
            }
        } else {
            UiSnapshot::default()
        };

        Ok(Some(WorkspaceSnapshot {
            workspace_id,
            path,
            name,
            branch,
            tabs,
            env,
            ui,
            saved_at,
        }))
    }

    pub async fn list_recent(&self, limit: usize) -> Result<Vec<WorkspaceSummary>> {
        let rows = sqlx::query(
            "SELECT id, path, name, branch, last_opened_at FROM workspaces ORDER BY last_opened_at DESC LIMIT ?",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("list recent: {e}")))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.get("id");
            let last_opened: String = row.get("last_opened_at");
            result.push(WorkspaceSummary {
                id: WorkspaceId::from_uuid(
                    Uuid::parse_str(&id)
                        .map_err(|e| Error::Database(format!("invalid workspace uuid: {e}")))?,
                ),
                path: row.get("path"),
                name: row.get("name"),
                branch: row.get("branch"),
                last_opened_at: DateTime::parse_from_rfc3339(&last_opened)
                    .map_err(|e| Error::Database(format!("invalid timestamp: {e}")))?
                    .with_timezone(&Utc),
            });
        }
        Ok(result)
    }

    pub async fn set_env(&self, workspace_id: WorkspaceId, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO workspace_env (workspace_id, key, value) VALUES (?, ?, ?)",
        )
        .bind(workspace_id.as_uuid().to_string())
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("set env: {e}")))?;
        Ok(())
    }

    pub async fn remove_env(&self, workspace_id: WorkspaceId, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM workspace_env WHERE workspace_id = ? AND key = ?")
            .bind(workspace_id.as_uuid().to_string())
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("remove env: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::id_from_path;

    #[tokio::test]
    async fn snapshot_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().join("ws.db");
        let store = WorkspaceStore::open(&db).await.expect("open");
        let ws_id = id_from_path(dir.path());

        let snapshot = WorkspaceSnapshot {
            workspace_id: ws_id,
            path: dir.path().display().to_string(),
            name: "test".to_string(),
            branch: Some("main".to_string()),
            tabs: vec![TabSnapshot {
                id: TabId::new(),
                title: "Terminal 1".to_string(),
                cwd: dir.path().display().to_string(),
                position: 0,
            }],
            env: HashMap::from([("FOO".to_string(), "bar".to_string())]),
            ui: UiSnapshot::default(),
            saved_at: Utc::now(),
        };

        store.save_snapshot(&snapshot).await.expect("save");
        let loaded = store
            .load_snapshot(ws_id)
            .await
            .expect("load")
            .expect("snapshot");
        assert_eq!(loaded.name, "test");
        assert_eq!(loaded.env.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(loaded.tabs.len(), 1);
    }
}
