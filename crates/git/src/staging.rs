use git2::{Repository, Status, StatusOptions};
use terminalos_shared::{Error, Result};

use crate::GitRepository;

/// A changed file in the working tree.
#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
}

/// Lists all changed files with staging state.
pub fn list_changed(repo: &GitRepository) -> Result<Vec<ChangedFile>> {
    let git_repo = open_repo(repo)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .renames_head_to_index(true)
        .renames_index_to_workdir(true);

    let statuses = git_repo
        .statuses(Some(&mut opts))
        .map_err(|e| Error::Git(format!("status failed: {e}")))?;

    let mut files = Vec::new();
    for entry in statuses.iter() {
        let Some(path) = entry.path() else {
            continue;
        };
        let flags = entry.status();
        files.push(ChangedFile {
            path: path.to_string(),
            staged: flags.is_index_new()
                || flags.is_index_modified()
                || flags.is_index_deleted()
                || flags.is_index_renamed(),
            unstaged: flags.is_wt_modified() || flags.is_wt_deleted() || flags.is_wt_renamed(),
            untracked: flags == Status::WT_NEW,
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

#[must_use]
pub fn format_changed(files: &[ChangedFile]) -> String {
    if files.is_empty() {
        return "No changed files.".to_string();
    }
    let mut out = String::new();
    for file in files {
        let mut tags = Vec::new();
        if file.staged {
            tags.push("staged");
        }
        if file.unstaged {
            tags.push("unstaged");
        }
        if file.untracked {
            tags.push("untracked");
        }
        out.push_str(&format!("- {} [{}]\n", file.path, tags.join(", ")));
    }
    out
}

/// Builds a `git add` command for the given paths.
#[must_use]
pub fn stage_command(paths: &[String]) -> String {
    if paths.is_empty() {
        "git add -A".to_string()
    } else {
        format!("git add {}", paths.join(" "))
    }
}

/// Builds a `git reset` command to unstage paths.
#[must_use]
pub fn unstage_command(paths: &[String]) -> String {
    if paths.is_empty() {
        "git reset HEAD".to_string()
    } else {
        format!("git reset HEAD -- {}", paths.join(" "))
    }
}

fn open_repo(repo: &GitRepository) -> Result<Repository> {
    Repository::open(repo.path()).map_err(|e| Error::Git(format!("open failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn stage_command_formats_paths() {
        assert_eq!(
            stage_command(&["a.rs".to_string(), "b.rs".to_string()]),
            "git add a.rs b.rs"
        );
    }

    #[test]
    fn lists_untracked_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(dir.path())
            .output()
            .expect("init");
        std::fs::write(dir.path().join("new.txt"), "x").expect("write");
        let repo = GitRepository::open(dir.path()).expect("open");
        let files = list_changed(&repo).expect("list");
        assert!(files.iter().any(|f| f.path == "new.txt" && f.untracked));
    }
}
