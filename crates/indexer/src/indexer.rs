use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use rayon::prelude::*;
use terminalos_search::{
    CodeParser, EmbeddingClient, EmbeddingConfig, HybridSearchConfig, SearchEngine, VectorStore,
};
use terminalos_shared::Result;

/// Statistics from an indexing run.
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub bytes_indexed: u64,
    pub skipped: usize,
    pub semantic_chunks: usize,
}

/// Indexes project files into keyword and semantic search stores.
pub struct ProjectIndexer {
    root: PathBuf,
    index_path: PathBuf,
    semantic_db: PathBuf,
    embedding: EmbeddingConfig,
    extensions: Vec<String>,
}

impl ProjectIndexer {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, index_path: impl Into<PathBuf>) -> Self {
        let index_path = index_path.into();
        let semantic_db = semantic_db_for_index(&index_path);
        Self {
            root: root.into(),
            index_path,
            semantic_db,
            embedding: EmbeddingConfig::default(),
            extensions: default_extensions(),
        }
    }

    #[must_use]
    pub fn with_semantic_db(
        root: impl Into<PathBuf>,
        index_path: impl Into<PathBuf>,
        semantic_db: impl Into<PathBuf>,
    ) -> Self {
        let mut indexer = Self::new(root, index_path);
        indexer.semantic_db = semantic_db.into();
        indexer
    }

    pub fn with_embedding_config(mut self, embedding: EmbeddingConfig) -> Self {
        self.embedding = embedding;
        self
    }

    pub fn index_all(&self) -> Result<IndexStats> {
        let files: Vec<PathBuf> = WalkBuilder::new(&self.root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .build()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
            .map(|e| e.into_path())
            .filter(|p| self.should_index(p))
            .collect();

        let results: Vec<(PathBuf, String)> = files
            .par_iter()
            .filter_map(|path| {
                let content = std::fs::read_to_string(path).ok()?;
                Some((path.clone(), content))
            })
            .collect();

        let mut engine = SearchEngine::open(&self.index_path)?;
        let mut stats = IndexStats::default();
        let mut parser = CodeParser::new();
        let mut all_chunks = Vec::new();

        for (path, content) in results {
            let rel = path
                .strip_prefix(&self.root)
                .unwrap_or(&path)
                .display()
                .to_string();
            stats.bytes_indexed += content.len() as u64;
            engine.index_document(&rel, &content)?;
            stats.files_indexed += 1;

            let chunks = parser.extract_chunks(&rel, &content);
            all_chunks.extend(chunks);
        }

        stats.skipped = files.len().saturating_sub(stats.files_indexed);
        stats.semantic_chunks = self.index_semantic_chunks(&all_chunks).unwrap_or(0);
        Ok(stats)
    }

    fn index_semantic_chunks(&self, chunks: &[terminalos_search::CodeChunk]) -> Result<usize> {
        if chunks.is_empty() {
            return Ok(0);
        }

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| terminalos_shared::Error::Search(format!("tokio runtime: {e}")))?;

        runtime.block_on(async {
            let store = VectorStore::open(&self.semantic_db).await?;
            store.clear().await?;
            let client = EmbeddingClient::new(self.embedding.clone());
            store.index_chunks(chunks, &client).await
        })
    }

    fn should_index(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| self.extensions.iter().any(|e| e == ext))
    }
}

#[must_use]
pub fn semantic_db_for_index(index_path: &Path) -> PathBuf {
    index_path
        .parent()
        .unwrap_or_else(|| Path::new(".terminalos"))
        .join("semantic.db")
}

#[must_use]
pub fn hybrid_config_from_embedding(embedding: EmbeddingConfig) -> HybridSearchConfig {
    HybridSearchConfig {
        embedding,
        ..HybridSearchConfig::default()
    }
}

fn default_extensions() -> Vec<String> {
    vec![
        "rs".into(),
        "toml".into(),
        "md".into(),
        "json".into(),
        "yaml".into(),
        "yml".into(),
        "ts".into(),
        "tsx".into(),
        "js".into(),
        "jsx".into(),
        "py".into(),
        "go".into(),
    ]
}
