use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use terminalos_shared::{Error, Result};

use crate::chunk::CodeChunk;
use crate::embeddings::{EmbeddingClient, cosine_similarity};

/// A semantic search hit with vector similarity score.
#[derive(Debug, Clone)]
pub struct SemanticHit {
    pub path: String,
    pub content: String,
    pub score: f32,
    pub symbol: Option<String>,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
}

/// SQLite-backed vector store for code chunk embeddings.
pub struct VectorStore {
    pool: SqlitePool,
}

impl VectorStore {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Database(format!("create vector dir: {e}")))?;
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
            CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY NOT NULL,
                path TEXT NOT NULL,
                symbol TEXT,
                kind TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                content TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS embeddings (
                chunk_id TEXT PRIMARY KEY NOT NULL,
                model TEXT NOT NULL,
                dimensions INTEGER NOT NULL,
                vector BLOB NOT NULL,
                FOREIGN KEY(chunk_id) REFERENCES chunks(id)
            );
            CREATE INDEX IF NOT EXISTS idx_chunks_path ON chunks(path);
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("vector migration failed: {e}")))?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        sqlx::query("DELETE FROM embeddings")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("clear embeddings: {e}")))?;
        sqlx::query("DELETE FROM chunks")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("clear chunks: {e}")))?;
        Ok(())
    }

    pub async fn upsert_chunks_with_embeddings(
        &self,
        chunks: &[CodeChunk],
        model: &str,
        vectors: &[Vec<f32>],
    ) -> Result<()> {
        if chunks.len() != vectors.len() {
            return Err(Error::Search(
                "chunk and embedding count mismatch".to_string(),
            ));
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Database(format!("begin tx: {e}")))?;

        for (chunk, vector) in chunks.iter().zip(vectors.iter()) {
            sqlx::query(
                "INSERT OR REPLACE INTO chunks (id, path, symbol, kind, start_line, end_line, content) VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&chunk.id)
            .bind(&chunk.path)
            .bind(&chunk.symbol)
            .bind(&chunk.kind)
            .bind(i64::from(chunk.start_line))
            .bind(i64::from(chunk.end_line))
            .bind(&chunk.content)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(format!("upsert chunk: {e}")))?;

            let blob = vector_to_blob(vector);
            sqlx::query(
                "INSERT OR REPLACE INTO embeddings (chunk_id, model, dimensions, vector) VALUES (?, ?, ?, ?)",
            )
            .bind(&chunk.id)
            .bind(model)
            .bind(vector.len() as i64)
            .bind(blob)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(format!("upsert embedding: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| Error::Database(format!("commit: {e}")))?;
        Ok(())
    }

    pub async fn index_chunks(
        &self,
        chunks: &[CodeChunk],
        client: &EmbeddingClient,
    ) -> Result<usize> {
        if chunks.is_empty() {
            return Ok(0);
        }

        let texts: Vec<String> = chunks.iter().map(embedding_text).collect();
        let vectors = client.embed_batch(&texts).await?;
        self.upsert_chunks_with_embeddings(chunks, &client.config().model, &vectors)
            .await?;
        Ok(chunks.len())
    }

    pub async fn semantic_search(
        &self,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<SemanticHit>> {
        let rows = sqlx::query(
            r"
            SELECT c.path, c.symbol, c.kind, c.start_line, c.end_line, c.content, e.vector
            FROM chunks c
            JOIN embeddings e ON e.chunk_id = c.id
            ",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("semantic query: {e}")))?;

        let mut hits = Vec::with_capacity(rows.len());
        for row in rows {
            let blob: Vec<u8> = row.get("vector");
            let vector = blob_to_vector(&blob)?;
            if vector.len() != query_vector.len() {
                continue;
            }

            let score = cosine_similarity(query_vector, &vector);
            hits.push(SemanticHit {
                path: row.get("path"),
                content: row.get("content"),
                score,
                symbol: row.get("symbol"),
                kind: row.get("kind"),
                start_line: row.get::<i64, _>("start_line") as u32,
                end_line: row.get::<i64, _>("end_line") as u32,
            });
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }

    pub async fn chunk_count(&self) -> Result<usize> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM chunks")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("count chunks: {e}")))?;
        Ok(row.get::<i64, _>("count") as usize)
    }
}

#[must_use]
pub fn embedding_text(chunk: &CodeChunk) -> String {
    if let Some(symbol) = &chunk.symbol {
        format!("{symbol}\n{}", chunk.content)
    } else {
        chunk.content.clone()
    }
}

fn vector_to_blob(vector: &[f32]) -> Vec<u8> {
    vector.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn blob_to_vector(blob: &[u8]) -> Result<Vec<f32>> {
    if blob.len() % 4 != 0 {
        return Err(Error::Search("invalid embedding blob length".to_string()));
    }

    let mut vector = Vec::with_capacity(blob.len() / 4);
    for chunk in blob.chunks_exact(4) {
        let bytes: [u8; 4] = chunk.try_into().expect("chunk size");
        vector.push(f32::from_le_bytes(bytes));
    }
    Ok(vector)
}

/// Merges semantic hits by chunk id for hybrid ranking.
#[must_use]
pub fn semantic_hit_key(hit: &SemanticHit) -> String {
    format!(
        "{}:{}:{}",
        hit.path,
        hit.start_line,
        hit.symbol.as_deref().unwrap_or("")
    )
}

/// Groups semantic hits by path for display.
#[must_use]
pub fn group_by_path(hits: &[SemanticHit]) -> HashMap<String, Vec<&SemanticHit>> {
    let mut grouped: HashMap<String, Vec<&SemanticHit>> = HashMap::new();
    for hit in hits {
        grouped.entry(hit.path.clone()).or_default().push(hit);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::CodeChunk;

    #[tokio::test]
    async fn vector_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = dir.path().join("semantic.db");
        let store = VectorStore::open(&db).await.expect("open");

        let chunk = CodeChunk::new(
            "src/lib.rs",
            Some("hello".to_string()),
            "function_item",
            1,
            3,
            "fn hello() {}",
        );
        let vector = vec![1.0, 0.0, 0.5];
        store
            .upsert_chunks_with_embeddings(&[chunk], "test-model", std::slice::from_ref(&vector))
            .await
            .expect("upsert");

        let hits = store.semantic_search(&vector, 5).await.expect("search");
        assert_eq!(hits.len(), 1);
        assert!((hits[0].score - 1.0).abs() < 0.01);
    }
}
