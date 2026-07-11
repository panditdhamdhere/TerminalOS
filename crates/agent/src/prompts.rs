/// System prompts for agent slash commands.
#[must_use]
pub fn agent_system_prompt() -> String {
    [
        "You are TerminalOS Coding Agent, an expert software engineer.",
        "You work inside the user's workspace and help edit, review, and improve code.",
        "When you need to use tools, respond with a single JSON object on its own line:",
        r#"{"tool":"read_file","path":"relative/path"}"#,
        r#"{"tool":"write_file","path":"relative/path","content":"full file content"}"#,
        r#"{"tool":"create_file","path":"relative/path","content":"full file content"}"#,
        r#"{"tool":"delete_file","path":"relative/path"}"#,
        r#"{"tool":"rename_file","from":"old","to":"new"}"#,
        r#"{"tool":"search","query":"search terms"}"#,
        r#"{"tool":"run_command","command":"cargo test -p crate"}"#,
        r#"{"tool":"finish","message":"your final response to the user"}"#,
        "Write and run_command actions require user confirmation before execution.",
        "Always use workspace-relative paths. Never use absolute paths or .. traversal.",
        "When editing a file, return the complete updated file content in write_file.",
    ]
    .join("\n")
}

#[must_use]
pub fn edit_prompt(path: &str, content: &str, instruction: &str) -> String {
    format!(
        "Edit `{path}`.\n\nInstruction: {instruction}\n\nCurrent file:\n```\n{content}\n```\n\nUse tools to read related files if needed, then write_file with the complete updated content, then finish with a summary."
    )
}

#[must_use]
pub fn create_prompt(path: &str, description: &str) -> String {
    format!(
        "Create new file `{path}`.\n\nDescription: {description}\n\nUse create_file with the full content, then finish with a summary."
    )
}

#[must_use]
pub fn fix_prompt(path: &str, content: &str, instruction: &str) -> String {
    format!(
        "Fix issues in `{path}`.\n\nTask: {instruction}\n\nCurrent file:\n```\n{content}\n```\n\nAnalyze, fix, write_file with the corrected content, then finish."
    )
}

#[must_use]
pub fn refactor_prompt(path: &str, content: &str, instruction: &str) -> String {
    format!(
        "Refactor `{path}`.\n\nInstruction: {instruction}\n\nCurrent file:\n```\n{content}\n```\n\nRefactor carefully, write_file with the result, then finish."
    )
}

#[must_use]
pub fn explain_prompt(path: &str, content: &str, question: &str) -> String {
    format!(
        "Explain `{path}`.\n\nQuestion: {question}\n\nFile content:\n```\n{content}\n```\n\nProvide a clear explanation. Use finish when done — no file changes needed."
    )
}

#[must_use]
pub fn review_prompt(path: &str, content: &str, git_context: &str) -> String {
    format!(
        "Review `{path}` for bugs, style, and improvements.\n\nGit context:\n{git_context}\n\nFile content:\n```\n{content}\n```\n\nProvide a structured code review. Use finish when done."
    )
}

#[must_use]
pub fn test_prompt(args: &str) -> String {
    let cmd = if args.trim().is_empty() {
        "cargo test --workspace".to_string()
    } else {
        format!("cargo test {args}")
    };
    format!(
        "Propose and run tests for this workspace.\n\nSuggested command: `{cmd}`\n\nUse run_command to propose the test command, then finish with guidance."
    )
}

#[must_use]
pub fn docs_prompt(path: &str, content: &str, focus: &str) -> String {
    format!(
        "Generate documentation for `{path}`.\n\nFocus: {focus}\n\nSource:\n```\n{content}\n```\n\nProduce markdown documentation. Use write_file to save as `{path}.md` or finish with the docs inline."
    )
}

#[must_use]
pub fn repo_analysis_prompt(workspace: &str, git_summary: &str, file_count: usize) -> String {
    format!(
        "Repository analysis for `{workspace}`.\n\nFiles indexed: {file_count}\nGit status:\n{git_summary}\n\nSummarize architecture, key crates, and suggested next steps. Use finish when done."
    )
}

#[must_use]
pub fn commit_message_prompt(staged_diff: &str, status_summary: &str) -> String {
    format!(
        "Generate a concise, conventional commit message for these staged changes.\n\nRepository status:\n{status_summary}\n\nStaged diff:\n```diff\n{staged_diff}\n```\n\nRespond with:\n1. A single-line commit message (max 72 chars)\n2. A blank line\n3. A detailed body explaining the why\n\nEnd with a fenced block:\n```commit\n<your commit message here>\n```"
    )
}

#[must_use]
pub fn pr_summary_prompt(base_ref: &str, commits: &str, diff: &str) -> String {
    format!(
        "Generate a pull request summary comparing against `{base_ref}`.\n\nCommits:\n{commits}\n\nDiff:\n```diff\n{diff}\n```\n\nProvide: title, summary, changes list, test plan, and risks."
    )
}

#[must_use]
pub fn diff_explain_prompt(diff: &str, path: Option<&str>) -> String {
    let scope = path
        .map(|p| format!("for `{p}`"))
        .unwrap_or_else(|| "for all changes".to_string());
    format!(
        "Explain the git diff {scope}.\n\n```diff\n{diff}\n```\n\nDescribe what changed, why it matters, and any concerns."
    )
}

#[must_use]
pub fn conflict_resolution_prompt(conflicts: &str) -> String {
    format!(
        "Help resolve these merge conflicts.\n\n{conflicts}\n\nFor each conflict, explain both sides and recommend a resolution with the merged content."
    )
}

#[must_use]
pub fn blame_explain_prompt(path: &str, blame: &str) -> String {
    format!(
        "Explain the git blame history for `{path}`.\n\n```\n{blame}\n```\n\nSummarize who changed what, when, and the evolution of this code."
    )
}

#[must_use]
pub fn health_recommendations_prompt(report: &str) -> String {
    format!(
        "Repository health report:\n\n{report}\n\nProvide actionable recommendations to improve repository hygiene."
    )
}
