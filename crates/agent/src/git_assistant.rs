use std::path::PathBuf;

use terminalos_git::{
    GitRepository, assess, blame_file, commit_log_against_ref, diff_against_ref, diff_for_path,
    find_conflicts, format_blame, format_changed, format_conflicts, format_diffs, format_report,
    list_changed, stage_command, staged_diff, unstage_command, unstaged_diff,
};
use terminalos_shared::Result;

use crate::AgentOutcome;
use crate::action::PendingAction;
use crate::prompts::{
    blame_explain_prompt, commit_message_prompt, conflict_resolution_prompt, diff_explain_prompt,
    health_recommendations_prompt, pr_summary_prompt,
};

/// Git assistant operations for slash commands.
pub struct GitAssistant {
    repo: GitRepository,
}

impl GitAssistant {
    pub fn open(workspace_root: &PathBuf) -> Result<Self> {
        let repo = GitRepository::discover(workspace_root)?;
        Ok(Self { repo })
    }

    pub fn handle_commit(&self) -> Result<AgentOutcome> {
        let staged = staged_diff(&self.repo)?;
        if staged.is_empty() {
            return Ok(AgentOutcome::Error(
                "Nothing staged. Use `/stage <path>` to stage files first.".to_string(),
            ));
        }
        let diff_text = format_diffs(&staged);
        let status = self.repo.status()?;
        let summary = format!(
            "branch: {}\nstaged: {}\nmodified: {}\nuntracked: {}",
            status.branch.unwrap_or_else(|| "unknown".to_string()),
            status.staged,
            status.modified,
            status.untracked
        );
        Ok(AgentOutcome::StartChat(commit_message_prompt(
            &diff_text, &summary,
        )))
    }

    pub fn handle_pr(&self, base_ref: &str) -> Result<AgentOutcome> {
        let commits = commit_log_against_ref(&self.repo, base_ref, 30)?;
        let diff = diff_against_ref(&self.repo, base_ref)?;
        Ok(AgentOutcome::StartChat(pr_summary_prompt(
            base_ref, &commits, &diff,
        )))
    }

    pub fn handle_diff(&self, path: Option<&str>) -> Result<AgentOutcome> {
        let diff = match path {
            Some(p) => diff_for_path(&self.repo, p)?,
            None => {
                let staged = format_diffs(&staged_diff(&self.repo)?);
                let unstaged = format_diffs(&unstaged_diff(&self.repo)?);
                if staged.is_empty() && unstaged.is_empty() {
                    return Ok(AgentOutcome::Message(
                        "No changes in working tree.".to_string(),
                    ));
                }
                format!("## Staged\n{staged}\n## Unstaged\n{unstaged}")
            }
        };
        Ok(AgentOutcome::StartChat(diff_explain_prompt(&diff, path)))
    }

    pub fn handle_conflict(&self, path: Option<&str>) -> Result<AgentOutcome> {
        let mut conflicts = find_conflicts(&self.repo)?;
        if let Some(p) = path {
            conflicts.retain(|c| c.path == p);
        }
        if conflicts.is_empty() {
            return Ok(AgentOutcome::Message(
                "No merge conflicts found.".to_string(),
            ));
        }
        let text = format_conflicts(&conflicts);
        Ok(AgentOutcome::StartChat(conflict_resolution_prompt(&text)))
    }

    pub fn handle_stage(&self, paths: &[String], unstage: bool) -> Result<AgentOutcome> {
        let command = if unstage {
            unstage_command(paths)
        } else {
            stage_command(paths)
        };
        Ok(AgentOutcome::Pending(PendingAction::run_command(command)))
    }

    pub fn handle_stage_list(&self) -> Result<AgentOutcome> {
        let files = list_changed(&self.repo)?;
        let listing = format_changed(&files);
        let help = "\nUse `/stage <path>` to stage or `/unstage <path>` to unstage.";
        Ok(AgentOutcome::Message(format!("{listing}{help}")))
    }

    pub fn handle_blame(&self, path: &str, line: Option<u32>) -> Result<AgentOutcome> {
        let start = line;
        let end = line.map(|l| l + 20);
        let entries = blame_file(&self.repo, path, start, end)?;
        if entries.is_empty() {
            return Ok(AgentOutcome::Error(format!("No blame data for `{path}`.")));
        }
        let blame = format_blame(&entries);
        Ok(AgentOutcome::StartChat(blame_explain_prompt(path, &blame)))
    }

    pub fn handle_health(&self) -> Result<AgentOutcome> {
        let report = assess(&self.repo)?;
        let formatted = format_report(&report);
        if report.has_failures() || report.has_warnings() {
            Ok(AgentOutcome::StartChat(health_recommendations_prompt(
                &formatted,
            )))
        } else {
            Ok(AgentOutcome::Message(formatted))
        }
    }

    pub fn propose_commit(&self, message: &str) -> AgentOutcome {
        let escaped = message.replace('\'', "'\\''");
        AgentOutcome::Pending(PendingAction::run_command(format!(
            "git commit -m '{escaped}'"
        )))
    }
}

/// Extracts a commit message from a fenced ```commit block.
#[must_use]
pub fn extract_commit_message(text: &str) -> Option<String> {
    let start = text.find("```commit")?;
    let rest = &text[start + 9..];
    let end = rest.find("```")?;
    let message = rest[..end].trim();
    if message.is_empty() {
        None
    } else {
        Some(message.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_commit_block() {
        let text = "Here is the message:\n```commit\nfeat: add git assistant\n\nDetails here.\n```";
        let msg = extract_commit_message(text).expect("commit");
        assert!(msg.contains("feat: add git assistant"));
    }
}
