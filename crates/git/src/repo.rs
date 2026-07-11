use std::path::{Path, PathBuf};

use git2::{Repository, Status, StatusOptions};
use terminalos_shared::{Error, Result};

/// Summary of repository working tree status.
#[derive(Debug, Clone, Default)]
pub struct RepoStatus {
    pub branch: Option<String>,
    pub ahead: i32,
    pub behind: i32,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub is_clean: bool,
}

/// Git repository wrapper.
#[derive(Debug, Clone)]
pub struct GitRepository {
    path: PathBuf,
}

impl GitRepository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        Repository::open(&path).map_err(|e| Error::Git(format!("open failed: {e}")))?;
        Ok(Self { path })
    }

    pub fn discover(path: impl AsRef<Path>) -> Result<Self> {
        let repo = Repository::discover(path.as_ref())
            .map_err(|e| Error::Git(format!("discover failed: {e}")))?;
        Ok(Self {
            path: repo.path().parent().unwrap_or(repo.path()).to_path_buf(),
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn status(&self) -> Result<RepoStatus> {
        let repo =
            Repository::open(&self.path).map_err(|e| Error::Git(format!("open failed: {e}")))?;

        let mut status = RepoStatus::default();

        if let Ok(head) = repo.head() {
            if let Some(name) = head.shorthand() {
                status.branch = Some(name.to_string());
            }
        }

        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|e| Error::Git(format!("status failed: {e}")))?;

        for entry in statuses.iter() {
            let flags = entry.status();
            if flags.is_index_new() || flags.is_index_modified() || flags.is_index_deleted() {
                status.staged += 1;
            }
            if flags.is_wt_modified() || flags.is_wt_deleted() {
                status.modified += 1;
            }
            if flags == Status::WT_NEW {
                status.untracked += 1;
            }
        }

        status.is_clean = status.staged == 0 && status.modified == 0 && status.untracked == 0;
        Ok(status)
    }
}
