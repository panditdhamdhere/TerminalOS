use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;
use terminalos_config::SearchConfig;
use terminalos_filesystem::FileOps;
use terminalos_git::GitRepository;
use terminalos_indexer::{ProjectIndexer, semantic_db_for_index};
use terminalos_search::{
    EmbeddingConfig, HybridSearchConfig, HybridSearchEngine, SearchMode, SearchQuery,
};
use terminalos_shared::{Error, Result};

use crate::action::PendingAction;

/// Parsed tool invocation from agent model output.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "tool", rename_all = "snake_case")]
pub enum ToolCall {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    CreateFile { path: String, content: String },
    DeleteFile { path: String },
    RenameFile { from: String, to: String },
    Search { query: String },
    RunCommand { command: String },
    Finish { message: String },
}

/// Result of executing a tool.
#[derive(Debug, Clone)]
pub enum ToolResult {
    Output(String),
    Pending(PendingAction),
    Finished(String),
}

/// Executes agent tools against the workspace.
pub struct ToolExecutor {
    file_ops: FileOps,
    index_path: PathBuf,
    semantic_db: PathBuf,
    search_config: HybridSearchConfig,
}

impl ToolExecutor {
    #[must_use]
    pub fn new(workspace_root: PathBuf, index_path: PathBuf, search_config: SearchConfig) -> Self {
        Self {
            file_ops: FileOps::new(workspace_root),
            semantic_db: semantic_db_for_index(&index_path),
            search_config: hybrid_config_from_search(search_config),
            index_path,
        }
    }

    #[must_use]
    pub fn file_ops(&self) -> &FileOps {
        &self.file_ops
    }

    pub fn execute(&mut self, call: &ToolCall) -> Result<ToolResult> {
        match call {
            ToolCall::ReadFile { path } => {
                let content = self.file_ops.read(path)?;
                Ok(ToolResult::Output(format!(
                    "Contents of {path}:\n{content}"
                )))
            }
            ToolCall::WriteFile { path, content } => {
                let preview = file_preview(path, content);
                let old = self.file_ops.read(path).unwrap_or_default();
                let diff = simple_diff(&old, content);
                let preview = if diff.is_empty() {
                    preview
                } else {
                    format!("{preview}\n\nDiff:\n{diff}")
                };
                Ok(ToolResult::Pending(PendingAction::write_file(
                    path.clone(),
                    preview,
                    content.clone(),
                )))
            }
            ToolCall::CreateFile { path, content } => {
                let preview = file_preview(path, content);
                Ok(ToolResult::Pending(PendingAction::create_file(
                    path.clone(),
                    preview,
                    content.clone(),
                )))
            }
            ToolCall::DeleteFile { path } => Ok(ToolResult::Pending(PendingAction::delete_file(
                path.clone(),
            ))),
            ToolCall::RenameFile { from, to } => Ok(ToolResult::Pending(
                PendingAction::rename_file(from.clone(), to.clone()),
            )),
            ToolCall::Search { query } => {
                let hits = self.search(query, 10)?;
                if hits.is_empty() {
                    return Ok(ToolResult::Output(format!("No results for '{query}'.")));
                }
                let mut out = format!("Search results for '{query}':\n");
                for hit in hits {
                    let location = match (hit.symbol.as_deref(), hit.start_line) {
                        (Some(symbol), Some(line)) => format!("{symbol}:{line}"),
                        (_, Some(line)) => format!("line {line}"),
                        _ => String::new(),
                    };
                    let label = if location.is_empty() {
                        hit.path.clone()
                    } else {
                        format!("{} ({location})", hit.path)
                    };
                    out.push_str(&format!(
                        "\n- {} [{:?}] (score {:.2})\n",
                        label,
                        hit.match_type.as_deref().unwrap_or("keyword"),
                        hit.score
                    ));
                    let snippet: String = hit.content.chars().take(200).collect();
                    out.push_str(&snippet);
                    if hit.content.len() > 200 {
                        out.push_str("...");
                    }
                    out.push('\n');
                }
                Ok(ToolResult::Output(out))
            }
            ToolCall::RunCommand { command } => Ok(ToolResult::Pending(
                PendingAction::run_command(command.clone()),
            )),
            ToolCall::Finish { message } => Ok(ToolResult::Finished(message.clone())),
        }
    }

    pub fn apply_pending(&self, action: &PendingAction) -> Result<String> {
        match action.kind {
            crate::action::ActionKind::WriteFile => {
                let path = action
                    .path
                    .as_ref()
                    .ok_or_else(|| Error::Filesystem("missing path".to_string()))?;
                let content = action
                    .content
                    .as_ref()
                    .ok_or_else(|| Error::Filesystem("missing content".to_string()))?;
                self.file_ops.write(path, content)?;
                Ok(format!("Wrote {path}"))
            }
            crate::action::ActionKind::CreateFile => {
                let path = action
                    .path
                    .as_ref()
                    .ok_or_else(|| Error::Filesystem("missing path".to_string()))?;
                let content = action
                    .content
                    .as_ref()
                    .ok_or_else(|| Error::Filesystem("missing content".to_string()))?;
                self.file_ops.create(path, content)?;
                Ok(format!("Created {path}"))
            }
            crate::action::ActionKind::DeleteFile => {
                let path = action
                    .path
                    .as_ref()
                    .ok_or_else(|| Error::Filesystem("missing path".to_string()))?;
                self.file_ops.delete(path)?;
                Ok(format!("Deleted {path}"))
            }
            crate::action::ActionKind::RenameFile => {
                let from = action
                    .path
                    .as_ref()
                    .ok_or_else(|| Error::Filesystem("missing source".to_string()))?;
                let to = action
                    .destination
                    .as_ref()
                    .ok_or_else(|| Error::Filesystem("missing destination".to_string()))?;
                self.file_ops.rename(from, to)?;
                Ok(format!("Renamed {from} → {to}"))
            }
            crate::action::ActionKind::RunCommand => {
                let command = action
                    .command
                    .as_ref()
                    .ok_or_else(|| Error::Terminal("missing command".to_string()))?;
                Ok(format!("COMMAND:{command}"))
            }
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<terminalos_search::SearchHit>> {
        self.ensure_index()?;
        let runtime =
            tokio::runtime::Runtime::new().map_err(|e| Error::Ai(format!("tokio runtime: {e}")))?;
        let engine = HybridSearchEngine::new(
            &self.index_path,
            &self.semantic_db,
            self.search_config.clone(),
        );
        runtime.block_on(engine.search(&SearchQuery {
            text: query.to_string(),
            limit,
        }))
    }

    pub fn ensure_index(&self) -> Result<()> {
        if self.index_path.join("meta.json").exists() {
            return Ok(());
        }
        let embedding = EmbeddingConfig {
            base_url: self.search_config.embedding.base_url.clone(),
            model: self.search_config.embedding.model.clone(),
            api_key_env: self.search_config.embedding.api_key_env.clone(),
        };
        let indexer = ProjectIndexer::new(self.file_ops.root(), &self.index_path)
            .with_embedding_config(embedding);
        let _ = indexer.index_all()?;
        Ok(())
    }

    #[must_use]
    pub fn git_summary(&self) -> String {
        match GitRepository::discover(self.file_ops.root()) {
            Ok(repo) => match repo.status() {
                Ok(status) => format!(
                    "branch: {}\nstaged: {}\nmodified: {}\nuntracked: {}\nclean: {}",
                    status.branch.unwrap_or_else(|| "unknown".to_string()),
                    status.staged,
                    status.modified,
                    status.untracked,
                    status.is_clean
                ),
                Err(e) => format!("git status error: {e}"),
            },
            Err(e) => format!("not a git repo: {e}"),
        }
    }
}

fn hybrid_config_from_search(config: SearchConfig) -> HybridSearchConfig {
    HybridSearchConfig {
        mode: match config.mode {
            terminalos_config::SearchMode::Hybrid => SearchMode::Hybrid,
            terminalos_config::SearchMode::Keyword => SearchMode::Keyword,
            terminalos_config::SearchMode::Semantic => SearchMode::Semantic,
        },
        keyword_weight: config.keyword_weight,
        semantic_weight: config.semantic_weight,
        embedding: EmbeddingConfig {
            base_url: config.embedding_base_url,
            model: config.embedding_model,
            api_key_env: config.embedding_api_key_env,
        },
    }
}

/// Extracts a JSON tool call from model output.
#[must_use]
pub fn extract_tool_call(text: &str) -> Option<ToolCall> {
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('{') {
            continue;
        }
        if let Ok(call) = serde_json::from_str::<ToolCall>(trimmed) {
            return Some(call);
        }
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            if let Ok(call) = serde_json::from_value::<ToolCall>(value) {
                return Some(call);
            }
        }
    }
    None
}

#[must_use]
pub fn file_preview(path: &str, content: &str) -> String {
    let lines: Vec<&str> = content.lines().take(12).collect();
    let mut preview = lines.join("\n");
    if content.lines().count() > 12 {
        preview.push_str("\n... (truncated)");
    }
    format!("{path}:\n{preview}")
}

#[must_use]
pub fn simple_diff(old: &str, new: &str) -> String {
    if old == new {
        return String::new();
    }
    let mut out = String::new();
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let max = old_lines.len().max(new_lines.len());
    for i in 0..max {
        let o = old_lines.get(i).copied().unwrap_or("");
        let n = new_lines.get(i).copied().unwrap_or("");
        if o != n {
            if !o.is_empty() {
                out.push_str(&format!("- {o}\n"));
            }
            if !n.is_empty() {
                out.push_str(&format!("+ {n}\n"));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_tool_call_json() {
        let text = r#"Let me read the file.
{"tool":"read_file","path":"src/lib.rs"}
"#;
        let call = extract_tool_call(text).expect("tool");
        assert!(matches!(call, ToolCall::ReadFile { .. }));
    }

    #[test]
    fn simple_diff_shows_changes() {
        let diff = simple_diff("a\nb", "a\nc");
        assert!(diff.contains("- b"));
        assert!(diff.contains("+ c"));
    }
}
