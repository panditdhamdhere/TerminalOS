use std::path::Path;

use terminalos_shared::{Error, Result};

use crate::GitRepository;

/// A merge conflict region in a file.
#[derive(Debug, Clone)]
pub struct ConflictRegion {
    pub start_line: u32,
    pub ours: String,
    pub theirs: String,
}

/// A file containing merge conflict markers.
#[derive(Debug, Clone)]
pub struct ConflictFile {
    pub path: String,
    pub regions: Vec<ConflictRegion>,
}

/// Scans the repository for merge conflict markers.
pub fn find_conflicts(repo: &GitRepository) -> Result<Vec<ConflictFile>> {
    let root = repo.path();
    let mut conflicts = Vec::new();
    scan_dir(root, root, &mut conflicts)?;
    Ok(conflicts)
}

fn scan_dir(root: &Path, dir: &Path, out: &mut Vec<ConflictFile>) -> Result<()> {
    if dir.file_name().is_some_and(|n| n == ".git") {
        return Ok(());
    }

    let entries =
        std::fs::read_dir(dir).map_err(|e| Error::Git(format!("read dir failed: {e}")))?;
    for entry in entries {
        let entry = entry.map_err(|e| Error::Git(format!("dir entry failed: {e}")))?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir(root, &path, out)?;
        } else if is_text_file(&path) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let regions = parse_conflict_markers(&content);
                if !regions.is_empty() {
                    let rel = path
                        .strip_prefix(root)
                        .unwrap_or(&path)
                        .display()
                        .to_string();
                    out.push(ConflictFile { path: rel, regions });
                }
            }
        }
    }
    Ok(())
}

fn is_text_file(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return true;
    };
    !matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "ico" | "woff" | "woff2" | "ttf" | "eot" | "zip" | "gz"
    )
}

fn parse_conflict_markers(content: &str) -> Vec<ConflictRegion> {
    let mut regions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("<<<<<<<") {
            let start = i as u32 + 1;
            i += 1;
            let mut ours = String::new();
            while i < lines.len() && !lines[i].starts_with("=======") {
                ours.push_str(lines[i]);
                ours.push('\n');
                i += 1;
            }
            i += 1; // skip =======
            let mut theirs = String::new();
            while i < lines.len() && !lines[i].starts_with(">>>>>>>") {
                theirs.push_str(lines[i]);
                theirs.push('\n');
                i += 1;
            }
            regions.push(ConflictRegion {
                start_line: start,
                ours,
                theirs,
            });
        }
        i += 1;
    }
    regions
}

#[must_use]
pub fn format_conflicts(files: &[ConflictFile]) -> String {
    if files.is_empty() {
        return "No merge conflicts found.".to_string();
    }
    let mut out = String::new();
    for file in files {
        out.push_str(&format!("## {}\n", file.path));
        for region in &file.regions {
            out.push_str(&format!("Conflict at line {}:\n", region.start_line));
            out.push_str("OURS:\n");
            out.push_str(&region.ours);
            out.push_str("THEIRS:\n");
            out.push_str(&region.theirs);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_conflict_markers() {
        let content = "line\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\n";
        let regions = parse_conflict_markers(content);
        assert_eq!(regions.len(), 1);
        assert!(regions[0].ours.contains("ours"));
        assert!(regions[0].theirs.contains("theirs"));
    }
}
