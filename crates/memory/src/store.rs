use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use terminalos_shared::{Error, Result, SessionId, WorkspaceId};
use uuid::Uuid;

/// Persisted workspace session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: SessionId,
    pub workspace_id: WorkspaceId,
    pub cwd: String,
    pub created_at: DateTime<Utc>,
}

/// Persisted AI conversation message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRecord {
    pub id: Uuid,
    pub session_id: SessionId,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// SQLite storage for sessions and conversations.
pub struct MemoryStore {
    pool: SqlitePool,
}

impl MemoryStore {
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
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY NOT NULL,
                workspace_id TEXT NOT NULL,
                cwd TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY NOT NULL,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            );
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("migration failed: {e}")))?;

        Ok(())
    }

    pub async fn save_session(&self, record: &SessionRecord) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO sessions (id, workspace_id, cwd, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(record.id.as_uuid().to_string())
        .bind(record.workspace_id.as_uuid().to_string())
        .bind(&record.cwd)
        .bind(record.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("save session failed: {e}")))?;

        Ok(())
    }

    pub async fn save_message(&self, record: &ConversationRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO conversations (id, session_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(record.id.to_string())
        .bind(record.session_id.as_uuid().to_string())
        .bind(&record.role)
        .bind(&record.content)
        .bind(record.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("save message failed: {e}")))?;

        Ok(())
    }

    pub async fn list_messages(&self, session_id: SessionId) -> Result<Vec<ConversationRecord>> {
        let rows = sqlx::query(
            "SELECT id, session_id, role, content, created_at FROM conversations WHERE session_id = ? ORDER BY created_at ASC",
        )
        .bind(session_id.as_uuid().to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("list messages failed: {e}")))?;

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.get("id");
            let session: String = row.get("session_id");
            let role: String = row.get("role");
            let content: String = row.get("content");
            let created_at: String = row.get("created_at");

            records.push(ConversationRecord {
                id: Uuid::parse_str(&id)
                    .map_err(|e| Error::Database(format!("invalid uuid: {e}")))?,
                session_id: SessionId::from_uuid(
                    Uuid::parse_str(&session)
                        .map_err(|e| Error::Database(format!("invalid session uuid: {e}")))?,
                ),
                role,
                content,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| Error::Database(format!("invalid timestamp: {e}")))?
                    .with_timezone(&Utc),
            });
        }

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn store_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.db");
        let store = MemoryStore::open(&db_path).await.expect("open");

        let session = SessionRecord {
            id: SessionId::new(),
            workspace_id: WorkspaceId::new(),
            cwd: "/tmp".to_string(),
            created_at: Utc::now(),
        };
        store.save_session(&session).await.expect("save session");

        let message = ConversationRecord {
            id: Uuid::new_v4(),
            session_id: session.id,
            role: "user".to_string(),
            content: "hello".to_string(),
            created_at: Utc::now(),
        };
        store.save_message(&message).await.expect("save message");

        let messages = store
            .list_messages(session.id)
            .await
            .expect("list messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "hello");
    }
}
