use git2::{BranchType, Repository, Status, StatusOptions};
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
    path: std::path::PathBuf,
}

impl GitRepository {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        Repository::open(&path).map_err(|e| Error::Git(format!("open failed: {e}")))?;
        Ok(Self { path })
    }

    pub fn discover(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let repo = Repository::discover(path.as_ref())
            .map_err(|e| Error::Git(format!("discover failed: {e}")))?;
        Ok(Self {
            path: repo
                .workdir()
                .or_else(|| repo.path().parent())
                .unwrap_or(repo.path())
                .to_path_buf(),
        })
    }

    #[must_use]
    pub fn path(&self) -> &std::path::Path {
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
            if self.has_upstream_internal(&repo, &head) {
                if let Ok((ahead, behind)) = ahead_behind(&repo, &head) {
                    status.ahead = ahead;
                    status.behind = behind;
                }
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

    /// Returns whether the current branch tracks a remote upstream.
    pub fn has_upstream(&self) -> bool {
        let Ok(repo) = Repository::open(&self.path) else {
            return false;
        };
        let Ok(head) = repo.head() else {
            return false;
        };
        self.has_upstream_internal(&repo, &head)
    }

    fn has_upstream_internal(&self, repo: &Repository, head: &git2::Reference) -> bool {
        let Some(name) = head.shorthand() else {
            return false;
        };
        repo.find_branch(name, BranchType::Local)
            .ok()
            .and_then(|b| b.upstream().ok())
            .is_some()
    }
}

fn ahead_behind(repo: &Repository, head: &git2::Reference) -> Result<(i32, i32)> {
    let branch = repo
        .find_branch(head.shorthand().unwrap_or("HEAD"), BranchType::Local)
        .ok();
    let Some(branch) = branch else {
        return Ok((0, 0));
    };
    let Ok(upstream) = branch.upstream() else {
        return Ok((0, 0));
    };
    let upstream_oid = upstream
        .get()
        .target()
        .ok_or_else(|| Error::Git("upstream has no target".to_string()))?;
    let head_oid = head
        .target()
        .ok_or_else(|| Error::Git("HEAD has no target".to_string()))?;
    let (ahead, behind) = repo
        .graph_ahead_behind(head_oid, upstream_oid)
        .map_err(|e| Error::Git(format!("ahead/behind failed: {e}")))?;
    Ok((ahead as i32, behind as i32))
}
