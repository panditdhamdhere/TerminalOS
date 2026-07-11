use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use rayon::prelude::*;
use terminalos_search::SearchEngine;
use terminalos_shared::Result;

/// Statistics from an indexing run.
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    pub files_indexed: usize,
    pub bytes_indexed: u64,
    pub skipped: usize,
}

/// Indexes project files into the search engine.
pub struct ProjectIndexer {
    root: PathBuf,
    index_path: PathBuf,
    extensions: Vec<String>,
}

impl ProjectIndexer {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>, index_path: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            index_path: index_path.into(),
            extensions: vec![
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
            ],
        }
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

        for (path, content) in results {
            let rel = path
                .strip_prefix(&self.root)
                .unwrap_or(&path)
                .display()
                .to_string();
            stats.bytes_indexed += content.len() as u64;
            engine.index_document(&rel, &content)?;
            stats.files_indexed += 1;
        }

        stats.skipped = files.len().saturating_sub(stats.files_indexed);
        Ok(stats)
    }

    fn should_index(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| self.extensions.iter().any(|e| e == ext))
    }
}
