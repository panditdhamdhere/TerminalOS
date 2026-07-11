use std::path::{Path, PathBuf};

use terminalos_shared::{Error, Result};

/// Workspace-scoped file operations with path sandboxing.
#[derive(Debug, Clone)]
pub struct FileOps {
    root: PathBuf,
}

impl FileOps {
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a relative path and rejects escapes outside the workspace root.
    pub fn resolve(&self, rel: &str) -> Result<PathBuf> {
        let rel = rel.trim().trim_start_matches("./");
        if rel.is_empty() {
            return Err(Error::Filesystem("empty path".to_string()));
        }
        if Path::new(rel).is_absolute() {
            return Err(Error::Filesystem(format!(
                "absolute paths not allowed: {rel}"
            )));
        }
        if rel.contains("..") {
            return Err(Error::Filesystem(format!(
                "path traversal not allowed: {rel}"
            )));
        }

        let joined = self.root.join(rel);
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());

        if joined.exists() {
            let canonical = joined
                .canonicalize()
                .map_err(|e| Error::Filesystem(format!("resolve failed: {e}")))?;
            if !canonical.starts_with(&root) {
                return Err(Error::Filesystem(format!("path escapes workspace: {rel}")));
            }
            return Ok(canonical);
        }

        let parent = joined
            .parent()
            .ok_or_else(|| Error::Filesystem(format!("invalid path: {rel}")))?;
        if parent.exists() {
            let canonical_parent = parent
                .canonicalize()
                .map_err(|e| Error::Filesystem(format!("resolve parent failed: {e}")))?;
            if !canonical_parent.starts_with(&root) {
                return Err(Error::Filesystem(format!("path escapes workspace: {rel}")));
            }
        } else if !self.path_stays_under_root(&joined, &root) {
            return Err(Error::Filesystem(format!("path escapes workspace: {rel}")));
        }

        Ok(joined)
    }

    fn path_stays_under_root(&self, path: &Path, root: &Path) -> bool {
        let mut current = path;
        while let Some(parent) = current.parent() {
            if parent == root || parent.starts_with(root) {
                return true;
            }
            if parent.exists() {
                return parent
                    .canonicalize()
                    .map(|p| p.starts_with(root))
                    .unwrap_or(false);
            }
            current = parent;
        }
        false
    }

    #[must_use]
    pub fn exists(&self, rel: &str) -> bool {
        self.resolve(rel).is_ok_and(|p| p.exists())
    }

    pub fn read(&self, rel: &str) -> Result<String> {
        let path = self.resolve(rel)?;
        if !path.is_file() {
            return Err(Error::Filesystem(format!("not a file: {rel}")));
        }
        std::fs::read_to_string(&path).map_err(|e| Error::Filesystem(format!("read failed: {e}")))
    }

    pub fn write(&self, rel: &str, content: &str) -> Result<()> {
        let path = self.resolve(rel)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Filesystem(format!("create parent failed: {e}")))?;
        }
        std::fs::write(&path, content).map_err(|e| Error::Filesystem(format!("write failed: {e}")))
    }

    pub fn create(&self, rel: &str, content: &str) -> Result<()> {
        let path = self.resolve(rel)?;
        if path.exists() {
            return Err(Error::Filesystem(format!("file already exists: {rel}")));
        }
        self.write(rel, content)
    }

    pub fn rename(&self, from: &str, to: &str) -> Result<()> {
        let from_path = self.resolve(from)?;
        let to_path = self.resolve(to)?;
        if !from_path.exists() {
            return Err(Error::Filesystem(format!("source not found: {from}")));
        }
        if to_path.exists() {
            return Err(Error::Filesystem(format!("destination exists: {to}")));
        }
        if let Some(parent) = to_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Filesystem(format!("create parent failed: {e}")))?;
        }
        std::fs::rename(&from_path, &to_path)
            .map_err(|e| Error::Filesystem(format!("rename failed: {e}")))
    }

    pub fn delete(&self, rel: &str) -> Result<()> {
        let path = self.resolve(rel)?;
        if !path.exists() {
            return Err(Error::Filesystem(format!("not found: {rel}")));
        }
        if path.is_dir() {
            std::fs::remove_dir_all(&path)
                .map_err(|e| Error::Filesystem(format!("delete dir failed: {e}")))?;
        } else {
            std::fs::remove_file(&path)
                .map_err(|e| Error::Filesystem(format!("delete file failed: {e}")))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ops = FileOps::new(dir.path());
        ops.write("hello.txt", "world").expect("write");
        let content = ops.read("hello.txt").expect("read");
        assert_eq!(content, "world");
    }

    #[test]
    fn rejects_path_traversal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ops = FileOps::new(dir.path());
        assert!(ops.read("../secret").is_err());
    }

    #[test]
    fn create_rejects_existing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ops = FileOps::new(dir.path());
        ops.write("a.txt", "x").expect("write");
        assert!(ops.create("a.txt", "y").is_err());
    }
}
