use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use terminalos_shared::{Error, Result};

/// A node in the workspace file tree.
#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

/// Builds a filtered file tree for the sidebar.
#[derive(Debug, Clone)]
pub struct FileTree {
    root: PathBuf,
}

impl FileTree {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Builds the file tree up to `max_depth` levels.
    pub fn build(&self, max_depth: usize) -> Result<FileNode> {
        let root = &self.root;
        if !root.exists() {
            return Err(Error::Filesystem(format!(
                "path does not exist: {}",
                root.display()
            )));
        }

        Self::build_node(root, max_depth)
    }

    fn build_node(path: &Path, depth: usize) -> Result<FileNode> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| Error::Filesystem(format!("metadata error: {e}")))?;

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let is_dir = metadata.is_dir();
        let mut children = Vec::new();

        if is_dir && depth > 0 {
            let mut entries: Vec<PathBuf> = WalkBuilder::new(path)
                .hidden(false)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                .max_depth(Some(1))
                .build()
                .filter_map(std::result::Result::ok)
                .filter(|entry| entry.path() != path)
                .map(|entry| entry.into_path())
                .collect();

            entries.sort_by(|a, b| {
                let a_dir = a.is_dir();
                let b_dir = b.is_dir();
                match (a_dir, b_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.file_name().cmp(&b.file_name()),
                }
            });

            for entry in entries {
                if let Ok(child) = Self::build_node(&entry, depth.saturating_sub(1)) {
                    children.push(child);
                }
            }
        }

        Ok(FileNode {
            path: path.to_path_buf(),
            name,
            is_dir,
            children,
        })
    }
}
