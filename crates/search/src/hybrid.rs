use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use terminalos_shared::Result;

use crate::embeddings::{EmbeddingClient, EmbeddingConfig};
use crate::engine::{SearchEngine, SearchHit, SearchQuery};
use crate::vector::VectorStore;

/// Search mode for the hybrid engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    #[default]
    Hybrid,
    Keyword,
    Semantic,
}

/// Hybrid search configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchConfig {
    pub mode: SearchMode,
    pub keyword_weight: f32,
    pub semantic_weight: f32,
    pub embedding: EmbeddingConfig,
}

impl Default for HybridSearchConfig {
    fn default() -> Self {
        Self {
            mode: SearchMode::Hybrid,
            keyword_weight: 0.4,
            semantic_weight: 0.6,
            embedding: EmbeddingConfig::default(),
        }
    }
}

/// Combined keyword + semantic search over a project index.
pub struct HybridSearchEngine {
    keyword_index: PathBuf,
    vector_db: PathBuf,
    config: HybridSearchConfig,
}

impl HybridSearchEngine {
    #[must_use]
    pub fn new(
        keyword_index: impl Into<PathBuf>,
        vector_db: impl Into<PathBuf>,
        config: HybridSearchConfig,
    ) -> Self {
        Self {
            keyword_index: keyword_index.into(),
            vector_db: vector_db.into(),
            config,
        }
    }

    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchHit>> {
        match self.config.mode {
            SearchMode::Keyword => self.keyword_search(query),
            SearchMode::Semantic => self.semantic_search(query).await,
            SearchMode::Hybrid => self.hybrid_search(query).await,
        }
    }

    fn keyword_search(&self, query: &SearchQuery) -> Result<Vec<SearchHit>> {
        let engine = SearchEngine::open(&self.keyword_index)?;
        let mut hits = engine.search(query)?;
        for hit in &mut hits {
            hit.match_type = Some("keyword".to_string());
        }
        Ok(hits)
    }

    async fn semantic_search(&self, query: &SearchQuery) -> Result<Vec<SearchHit>> {
        let store = VectorStore::open(&self.vector_db).await?;
        if store.chunk_count().await? == 0 {
            return Ok(Vec::new());
        }

        let client = EmbeddingClient::new(self.config.embedding.clone());
        let query_vector = client.embed(&query.text).await?;
        let semantic_hits = store.semantic_search(&query_vector, query.limit).await?;

        Ok(semantic_hits
            .into_iter()
            .map(|hit| SearchHit {
                path: hit.path,
                content: hit.content,
                score: hit.score,
                symbol: hit.symbol,
                kind: Some(hit.kind),
                start_line: Some(hit.start_line),
                end_line: Some(hit.end_line),
                match_type: Some("semantic".to_string()),
            })
            .collect())
    }

    async fn hybrid_search(&self, query: &SearchQuery) -> Result<Vec<SearchHit>> {
        let keyword_hits = self.keyword_search(query).unwrap_or_default();
        let semantic_hits = self.semantic_search(query).await.unwrap_or_default();

        if keyword_hits.is_empty() && semantic_hits.is_empty() {
            return Ok(Vec::new());
        }

        if semantic_hits.is_empty() {
            return Ok(keyword_hits);
        }

        if keyword_hits.is_empty() {
            return Ok(semantic_hits);
        }

        let max_keyword = keyword_hits
            .iter()
            .map(|hit| hit.score)
            .fold(0.0f32, f32::max)
            .max(1.0);

        let mut merged: HashMap<String, SearchHit> = HashMap::new();

        for hit in keyword_hits {
            let key = hit_key(&hit);
            let normalized = hit.score / max_keyword;
            let score = normalized * self.config.keyword_weight;
            merged.insert(
                key,
                SearchHit {
                    path: hit.path,
                    content: hit.content,
                    score,
                    symbol: hit.symbol,
                    kind: hit.kind,
                    start_line: hit.start_line,
                    end_line: hit.end_line,
                    match_type: Some("hybrid".to_string()),
                },
            );
        }

        for hit in semantic_hits {
            let key = hit_key(&hit);
            let semantic_score = hit.score * self.config.semantic_weight;
            merged
                .entry(key)
                .and_modify(|existing| {
                    existing.score += semantic_score;
                    if existing.content.len() < hit.content.len() {
                        existing.content = hit.content.clone();
                    }
                    if existing.symbol.is_none() {
                        existing.symbol = hit.symbol.clone();
                    }
                    if existing.kind.is_none() {
                        existing.kind = hit.kind.clone();
                    }
                    if existing.start_line.is_none() {
                        existing.start_line = hit.start_line;
                    }
                    if existing.end_line.is_none() {
                        existing.end_line = hit.end_line;
                    }
                })
                .or_insert(SearchHit {
                    path: hit.path,
                    content: hit.content,
                    score: semantic_score,
                    symbol: hit.symbol,
                    kind: hit.kind,
                    start_line: hit.start_line,
                    end_line: hit.end_line,
                    match_type: Some("hybrid".to_string()),
                });
        }

        let mut hits: Vec<SearchHit> = merged.into_values().collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(query.limit);
        Ok(hits)
    }
}

fn hit_key(hit: &SearchHit) -> String {
    if let (Some(start), Some(symbol)) = (hit.start_line, hit.symbol.as_deref()) {
        format!("{}:{start}:{symbol}", hit.path)
    } else if let Some(start) = hit.start_line {
        format!("{}:{start}", hit.path)
    } else {
        format!("{}:{}", hit.path, hit.content.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_prefers_hybrid() {
        let config = HybridSearchConfig::default();
        assert_eq!(config.mode, SearchMode::Hybrid);
        assert!(config.semantic_weight > config.keyword_weight);
    }
}
