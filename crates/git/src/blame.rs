use std::path::Path;

use git2::{BlameOptions, Repository};
use terminalos_shared::{Error, Result};

use crate::GitRepository;

/// A single blame line attribution.
#[derive(Debug, Clone)]
pub struct BlameEntry {
    pub line: u32,
    pub commit: String,
    pub author: String,
    pub date: String,
    pub content: String,
}

/// Returns blame information for a file, optionally limited to a line range.
pub fn blame_file(
    repo: &GitRepository,
    path: &str,
    start_line: Option<u32>,
    end_line: Option<u32>,
) -> Result<Vec<BlameEntry>> {
    let git_repo = open_repo(repo)?;
    let file_path = repo.path().join(path);
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| Error::Git(format!("read '{path}' failed: {e}")))?;
    let lines: Vec<&str> = content.lines().collect();

    let start = start_line.unwrap_or(1).max(1) as usize;
    let end = end_line
        .map(|e| e as usize)
        .unwrap_or(lines.len())
        .min(lines.len());

    let mut opts = BlameOptions::new();
    opts.min_line(start);
    opts.max_line(end);

    let blame = git_repo
        .blame_file(Path::new(path), Some(&mut opts))
        .map_err(|e| Error::Git(format!("blame failed for '{path}': {e}")))?;

    let mut entries = Vec::new();
    for line_no in start..=end {
        let Some(hunk) = blame.get_line(line_no) else {
            continue;
        };
        let commit = git_repo
            .find_commit(hunk.final_commit_id())
            .map_err(|e| Error::Git(format!("find commit failed: {e}")))?;
        let author = commit.author().name().unwrap_or("unknown").to_string();
        let time = commit.time().seconds();
        let date = format_timestamp(time);
        let commit_id: String = hunk.final_commit_id().to_string().chars().take(8).collect();

        entries.push(BlameEntry {
            line: line_no as u32,
            commit: commit_id,
            author,
            date,
            content: lines[line_no - 1].to_string(),
        });
    }

    Ok(entries)
}

#[must_use]
pub fn format_blame(entries: &[BlameEntry]) -> String {
    let mut out = String::new();
    for entry in entries {
        out.push_str(&format!(
            "L{}: {} {} <{}> {}\n",
            entry.line, entry.commit, entry.author, entry.date, entry.content
        ));
    }
    out
}

fn format_timestamp(secs: i64) -> String {
    let days = secs / 86_400;
    let years = 1970 + days / 365;
    let months = (days % 365) / 30 + 1;
    let day = (days % 30) + 1;
    format!("{years:04}-{months:02}-{day:02}")
}

fn open_repo(repo: &GitRepository) -> Result<Repository> {
    Repository::open(repo.path()).map_err(|e| Error::Git(format!("open failed: {e}")))
}
