use std::cell::{Cell, RefCell};

use git2::{Diff, DiffOptions, Repository};
use terminalos_shared::{Error, Result};

use crate::GitRepository;

/// A single diff line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub origin: char,
    pub content: String,
}

/// Per-file diff output.
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub status: char,
    pub patch: String,
}

/// Renders the staged diff (index vs HEAD).
pub fn staged_diff(repo: &GitRepository) -> Result<Vec<FileDiff>> {
    let git_repo = open_repo(repo)?;
    let head = git_repo.head().ok();
    let head_tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());

    let mut opts = DiffOptions::new();
    let diff = git_repo
        .diff_tree_to_index(head_tree.as_ref(), None, Some(&mut opts))
        .map_err(|e| Error::Git(format!("staged diff failed: {e}")))?;

    collect_diffs(&diff)
}

/// Renders unstaged changes (workdir vs index).
pub fn unstaged_diff(repo: &GitRepository) -> Result<Vec<FileDiff>> {
    let git_repo = open_repo(repo)?;
    let mut opts = DiffOptions::new();
    let diff = git_repo
        .diff_index_to_workdir(None, Some(&mut opts))
        .map_err(|e| Error::Git(format!("unstaged diff failed: {e}")))?;

    collect_diffs(&diff)
}

/// Combined staged + unstaged diff for a path filter.
pub fn diff_for_path(repo: &GitRepository, path: &str) -> Result<String> {
    let git_repo = open_repo(repo)?;
    let mut opts = DiffOptions::new();
    opts.pathspec(path);

    let head = git_repo.head().ok();
    let head_tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());

    let staged = git_repo
        .diff_tree_to_index(head_tree.as_ref(), None, Some(&mut opts))
        .map_err(|e| Error::Git(format!("path staged diff failed: {e}")))?;

    let mut unstaged_opts = DiffOptions::new();
    unstaged_opts.pathspec(path);
    let unstaged = git_repo
        .diff_index_to_workdir(None, Some(&mut unstaged_opts))
        .map_err(|e| Error::Git(format!("path unstaged diff failed: {e}")))?;

    let staged_text = format_diffs(&collect_diffs(&staged)?);
    let unstaged_text = format_diffs(&collect_diffs(&unstaged)?);

    if staged_text.is_empty() && unstaged_text.is_empty() {
        return Ok(format!("No changes for `{path}`."));
    }

    let mut out = String::new();
    if !staged_text.is_empty() {
        out.push_str("## Staged\n");
        out.push_str(&staged_text);
    }
    if !unstaged_text.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str("## Unstaged\n");
        out.push_str(&unstaged_text);
    }
    Ok(out)
}

/// Diff between current branch and a base ref (e.g. `main`).
pub fn diff_against_ref(repo: &GitRepository, base_ref: &str) -> Result<String> {
    let git_repo = open_repo(repo)?;
    let head = git_repo
        .head()
        .map_err(|e| Error::Git(format!("no HEAD: {e}")))?;
    let head_tree = head
        .peel_to_tree()
        .map_err(|e| Error::Git(format!("head tree failed: {e}")))?;

    let base = git_repo
        .revparse_single(base_ref)
        .map_err(|e| Error::Git(format!("unknown ref '{base_ref}': {e}")))?;
    let base_tree = base
        .peel_to_tree()
        .map_err(|e| Error::Git(format!("base tree failed: {e}")))?;

    let mut opts = DiffOptions::new();
    let diff = git_repo
        .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut opts))
        .map_err(|e| Error::Git(format!("branch diff failed: {e}")))?;

    let formatted = format_diffs(&collect_diffs(&diff)?);
    if formatted.is_empty() {
        Ok("No changes compared to base.".to_string())
    } else {
        Ok(formatted)
    }
}

/// Recent commit log for PR summaries.
pub fn commit_log_against_ref(
    repo: &GitRepository,
    base_ref: &str,
    limit: usize,
) -> Result<String> {
    let git_repo = open_repo(repo)?;
    let head = git_repo
        .head()
        .map_err(|e| Error::Git(format!("no HEAD: {e}")))?;
    let base = git_repo
        .revparse_single(base_ref)
        .map_err(|e| Error::Git(format!("unknown ref '{base_ref}': {e}")))?;

    let mut revwalk = git_repo
        .revwalk()
        .map_err(|e| Error::Git(format!("revwalk failed: {e}")))?;
    revwalk
        .push(head.target().expect("head oid"))
        .map_err(|e| Error::Git(format!("push head failed: {e}")))?;
    revwalk
        .hide(base.id())
        .map_err(|e| Error::Git(format!("hide base failed: {e}")))?;

    let mut out = String::new();
    let mut count = 0;
    for oid in revwalk {
        let oid = oid.map_err(|e| Error::Git(format!("revwalk iter failed: {e}")))?;
        let commit = git_repo
            .find_commit(oid)
            .map_err(|e| Error::Git(format!("find commit failed: {e}")))?;
        let summary = commit.summary().unwrap_or("(no message)");
        let author = commit.author().name().unwrap_or("unknown").to_string();
        out.push_str(&format!("- {} ({author})\n", summary));
        count += 1;
        if count >= limit {
            break;
        }
    }

    if out.is_empty() {
        Ok(format!("No commits ahead of {base_ref}."))
    } else {
        Ok(out)
    }
}

fn open_repo(repo: &GitRepository) -> Result<Repository> {
    Repository::open(repo.path()).map_err(|e| Error::Git(format!("open failed: {e}")))
}

fn collect_diffs(diff: &Diff) -> Result<Vec<FileDiff>> {
    let files = RefCell::new(Vec::<FileDiff>::new());
    let current_file = Cell::new(0_usize);

    diff.foreach(
        &mut |delta, _| {
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let old_path = delta
                .old_file()
                .path()
                .map(|p| p.to_string_lossy().to_string());
            let status = match delta.status() {
                git2::Delta::Added => 'A',
                git2::Delta::Deleted => 'D',
                git2::Delta::Modified => 'M',
                git2::Delta::Renamed => 'R',
                _ => '?',
            };
            files.borrow_mut().push(FileDiff {
                path,
                old_path,
                status,
                patch: String::new(),
            });
            current_file.set(files.borrow().len().saturating_sub(1));
            true
        },
        None,
        None,
        Some(&mut |_delta, _hunk, line| {
            if let Some(file) = files.borrow_mut().get_mut(current_file.get()) {
                let origin = line.origin();
                let content = std::str::from_utf8(line.content()).unwrap_or("");
                file.patch.push(origin);
                file.patch.push_str(content);
                if origin != ' ' && !content.ends_with('\n') {
                    file.patch.push('\n');
                }
            }
            true
        }),
    )
    .map_err(|e| Error::Git(format!("diff foreach failed: {e}")))?;

    Ok(files.into_inner())
}

#[must_use]
pub fn format_diffs(diffs: &[FileDiff]) -> String {
    let mut out = String::new();
    for file in diffs {
        out.push_str(&format!("diff --{} {}\n", file.status, file.path));
        if let Some(old) = &file.old_path {
            if old != &file.path {
                out.push_str(&format!("rename from {old}\n"));
                out.push_str(&format!("rename to {}\n", file.path));
            }
        }
        out.push_str(&file.patch);
        if !file.patch.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn init_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(dir)
            .output()
            .expect("git init");
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .expect("git config email");
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .expect("git config name");
        std::fs::write(dir.join("hello.txt"), "hello\n").expect("write");
        Command::new("git")
            .args(["add", "hello.txt"])
            .current_dir(dir)
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(dir)
            .output()
            .expect("git commit");
    }

    #[test]
    fn detects_staged_changes() {
        let dir = tempfile::tempdir().expect("tempdir");
        init_repo(dir.path());
        std::fs::write(dir.path().join("hello.txt"), "hello world\n").expect("write");
        Command::new("git")
            .args(["add", "hello.txt"])
            .current_dir(dir.path())
            .output()
            .expect("git add");

        let repo = GitRepository::open(dir.path()).expect("open");
        let diffs = staged_diff(&repo).expect("staged");
        assert!(!diffs.is_empty());
    }
}
