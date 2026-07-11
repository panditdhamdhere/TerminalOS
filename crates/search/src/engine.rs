use std::path::Path;

use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{STORED, Schema, TEXT, Value};
use tantivy::{Index, IndexWriter, ReloadPolicy, TantivyDocument, doc};
use terminalos_shared::{Error, Result};

/// A search query against the code index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub limit: usize,
}

/// A single search result hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub content: String,
    pub score: f32,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub start_line: Option<u32>,
    #[serde(default)]
    pub end_line: Option<u32>,
    #[serde(default)]
    pub match_type: Option<String>,
}

/// Tantivy-backed full-text search engine.
pub struct SearchEngine {
    index: Index,
    path_field: tantivy::schema::Field,
    content_field: tantivy::schema::Field,
}

impl SearchEngine {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let mut schema_builder = Schema::builder();
        let path_field = schema_builder.add_text_field("path", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT | STORED);
        let schema = schema_builder.build();

        let index = if path.join("meta.json").exists() {
            Index::open_in_dir(path).map_err(|e| Error::Search(format!("open index: {e}")))?
        } else {
            if !path.exists() {
                std::fs::create_dir_all(path)
                    .map_err(|e| Error::Search(format!("create index dir: {e}")))?;
            }
            Index::create_in_dir(path, schema)
                .map_err(|e| Error::Search(format!("create index: {e}")))?
        };

        Ok(Self {
            index,
            path_field,
            content_field,
        })
    }

    pub fn index_document(&mut self, path: &str, content: &str) -> Result<()> {
        let mut writer: IndexWriter = self
            .index
            .writer(50_000_000)
            .map_err(|e| Error::Search(format!("writer: {e}")))?;

        writer
            .add_document(doc!(
                self.path_field => path.to_string(),
                self.content_field => content.to_string(),
            ))
            .map_err(|e| Error::Search(format!("add doc: {e}")))?;

        writer
            .commit()
            .map_err(|e| Error::Search(format!("commit: {e}")))?;

        Ok(())
    }

    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchHit>> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| Error::Search(format!("reader: {e}")))?;

        let searcher = reader.searcher();
        let parser = QueryParser::for_index(&self.index, vec![self.content_field]);
        let parsed = parser
            .parse_query(&query.text)
            .map_err(|e| Error::Search(format!("parse query: {e}")))?;

        let top_docs = searcher
            .search(&parsed, &TopDocs::with_limit(query.limit))
            .map_err(|e| Error::Search(format!("search: {e}")))?;

        let mut hits = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| Error::Search(format!("doc: {e}")))?;

            let path = doc
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let content = doc
                .get_first(self.content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            hits.push(SearchHit {
                path,
                content,
                score,
                symbol: None,
                kind: None,
                start_line: None,
                end_line: None,
                match_type: None,
            });
        }

        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_and_search() {
        let dir = tempfile::tempdir().expect("tempdir");
        let index_path = dir.path().join("index");
        let mut engine = SearchEngine::open(&index_path).expect("open");
        engine
            .index_document("src/main.rs", "fn main terminalos")
            .expect("index");

        let hits = engine
            .search(&SearchQuery {
                text: "terminalos".to_string(),
                limit: 10,
            })
            .expect("search");

        assert!(!hits.is_empty());
    }
}
