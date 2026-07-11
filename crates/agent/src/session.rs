use std::path::PathBuf;

use terminalos_ai::{ChatEngine, ChatMessage, MessageRole};
use terminalos_config::{AgentConfig, PluginConfig, SearchConfig};
use terminalos_plugin::PluginManager;
use terminalos_shared::{Error, Result};
use tokio::runtime::Runtime;

use crate::action::PendingAction;
use crate::command::SlashCommand;
use crate::git_assistant::GitAssistant;
use crate::prompts::{
    agent_system_prompt, create_prompt, docs_prompt, edit_prompt, explain_prompt, fix_prompt,
    refactor_prompt, repo_analysis_prompt, review_prompt, test_prompt,
};
use crate::tools::{ToolExecutor, ToolResult, extract_tool_call};

/// Outcome of handling a slash command or agent step.
#[derive(Debug, Clone)]
pub enum AgentOutcome {
    /// Show a message directly in chat (no AI call).
    Message(String),
    /// Start streaming AI response via ChatEngine (single-shot).
    StartChat(String),
    /// Run multi-step agent tool loop with the given prompt.
    RunAgentLoop(String),
    /// Agent loop completed with a final message.
    Finished(String),
    /// Waiting for user to confirm a destructive action.
    Pending(PendingAction),
    /// Command was rejected or failed.
    Error(String),
}

/// Confirmed action execution result.
#[derive(Debug, Clone)]
pub enum ConfirmedResult {
    FileChanged(String),
    RunCommand(String),
}

/// Orchestrates slash commands, tool execution, and the agent loop.
pub struct AgentSession {
    executor: ToolExecutor,
    config: AgentConfig,
    plugin_config: PluginConfig,
    pending: Option<PendingAction>,
    workspace_name: String,
    workspace_root: PathBuf,
}

impl AgentSession {
    #[must_use]
    pub fn new(
        workspace_root: PathBuf,
        index_path: PathBuf,
        config: AgentConfig,
        search_config: SearchConfig,
        plugin_config: PluginConfig,
    ) -> Self {
        let workspace_name = workspace_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());

        Self {
            executor: ToolExecutor::new(workspace_root.clone(), index_path, search_config),
            config,
            plugin_config,
            workspace_name,
            pending: None,
            workspace_root,
        }
    }

    fn git(&self) -> Result<GitAssistant> {
        GitAssistant::open(&self.workspace_root)
    }

    #[must_use]
    pub fn pending(&self) -> Option<&PendingAction> {
        self.pending.as_ref()
    }

    pub fn clear_pending(&mut self) {
        self.pending = None;
    }

    pub fn reject_pending(&mut self) -> AgentOutcome {
        if self.pending.take().is_some() {
            AgentOutcome::Message("Action cancelled.".to_string())
        } else {
            AgentOutcome::Error("No pending action.".to_string())
        }
    }

    pub fn handle_command(&mut self, command: SlashCommand) -> Result<AgentOutcome> {
        match command {
            SlashCommand::Search { query } => self.handle_search(&query),
            SlashCommand::Analyze => self.analyze_repository(),
            SlashCommand::Explain { path, question } => self.handle_explain(&path, &question),
            SlashCommand::Review { path } => self.handle_review(&path),
            SlashCommand::Test { args } => Ok(AgentOutcome::RunAgentLoop(test_prompt(&args))),
            SlashCommand::Edit { path, instruction } => {
                self.handle_mutating(&path, &instruction, CommandMode::Edit)
            }
            SlashCommand::Fix { path, instruction } => {
                self.handle_mutating(&path, &instruction, CommandMode::Fix)
            }
            SlashCommand::Refactor { path, instruction } => {
                self.handle_mutating(&path, &instruction, CommandMode::Refactor)
            }
            SlashCommand::Create { path, description } => Ok(AgentOutcome::RunAgentLoop(
                create_prompt(&path, &description),
            )),
            SlashCommand::Docs { path, focus } => self.handle_docs(&path, &focus),
            SlashCommand::Commit => self.git()?.handle_commit(),
            SlashCommand::Pr { base } => self.git()?.handle_pr(&base),
            SlashCommand::Diff { path } => self.git()?.handle_diff(path.as_deref()),
            SlashCommand::Conflict { path } => self.git()?.handle_conflict(path.as_deref()),
            SlashCommand::Stage { paths } => {
                if paths.is_empty() {
                    self.git()?.handle_stage_list()
                } else {
                    self.git()?.handle_stage(&paths, false)
                }
            }
            SlashCommand::Unstage { paths } => self.git()?.handle_stage(&paths, true),
            SlashCommand::Blame { path, line } => self.git()?.handle_blame(&path, line),
            SlashCommand::Health => self.git()?.handle_health(),
            SlashCommand::Plugin {
                name,
                command,
                args,
            } => self.handle_plugin(&name, &command, &args),
        }
    }

    pub fn run_agent_loop(
        &mut self,
        chat: &ChatEngine,
        runtime: &Runtime,
        user_prompt: &str,
    ) -> Result<AgentOutcome> {
        let mut messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: agent_system_prompt(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: user_prompt.to_string(),
            },
        ];

        for _ in 0..self.config.max_iterations {
            let response = chat.complete_sync(&messages, runtime)?;
            let Some(call) = extract_tool_call(&response) else {
                return Ok(AgentOutcome::Finished(response));
            };

            match self.executor.execute(&call)? {
                ToolResult::Finished(message) => {
                    return Ok(AgentOutcome::Finished(message));
                }
                ToolResult::Pending(action) => {
                    self.pending = Some(action);
                    return Ok(AgentOutcome::Pending(
                        self.pending.clone().expect("pending"),
                    ));
                }
                ToolResult::Output(output) => {
                    messages.push(ChatMessage {
                        role: MessageRole::Assistant,
                        content: response,
                    });
                    messages.push(ChatMessage {
                        role: MessageRole::User,
                        content: format!("Tool result:\n{output}"),
                    });
                }
            }
        }

        Err(Error::Ai("agent exceeded max iterations".to_string()))
    }

    pub fn confirm_pending(&mut self) -> Result<ConfirmedResult> {
        let action = self
            .pending
            .take()
            .ok_or_else(|| Error::Ai("no pending action".to_string()))?;

        let result = self.executor.apply_pending(&action)?;
        if let Some(command) = action.command {
            if result.starts_with("COMMAND:") {
                return Ok(ConfirmedResult::RunCommand(command));
            }
        }
        Ok(ConfirmedResult::FileChanged(result))
    }

    pub fn analyze_repository(&mut self) -> Result<AgentOutcome> {
        self.executor.ensure_index()?;
        let git = self.executor.git_summary();
        let prompt = repo_analysis_prompt(&self.workspace_name, &git, 0);
        Ok(AgentOutcome::StartChat(prompt))
    }

    fn handle_search(&mut self, query: &str) -> Result<AgentOutcome> {
        let hits = self.executor.search(query, 15)?;
        if hits.is_empty() {
            return Ok(AgentOutcome::Message(format!("No results for '{query}'.")));
        }
        let mut out = format!("**Search results for** `{query}`:\n\n");
        for hit in hits {
            let location = match (hit.symbol.as_deref(), hit.start_line) {
                (Some(symbol), Some(line)) => format!("`{symbol}`:{line}"),
                (_, Some(line)) => format!("line {line}"),
                _ => String::new(),
            };
            let label = if location.is_empty() {
                format!("`{}`", hit.path)
            } else {
                format!("`{}` ({location})", hit.path)
            };
            out.push_str(&format!(
                "- {} [{:?}] (score {:.2})\n",
                label,
                hit.match_type.as_deref().unwrap_or("keyword"),
                hit.score
            ));
        }
        Ok(AgentOutcome::Message(out))
    }

    fn handle_explain(&self, path: &str, question: &str) -> Result<AgentOutcome> {
        let content = self.executor.file_ops().read(path)?;
        Ok(AgentOutcome::StartChat(explain_prompt(
            path, &content, question,
        )))
    }

    fn handle_review(&self, path: &str) -> Result<AgentOutcome> {
        let content = self.executor.file_ops().read(path)?;
        let git = self.executor.git_summary();
        Ok(AgentOutcome::StartChat(review_prompt(path, &content, &git)))
    }

    fn handle_docs(&self, path: &str, focus: &str) -> Result<AgentOutcome> {
        let content = self.executor.file_ops().read(path)?;
        Ok(AgentOutcome::StartChat(docs_prompt(path, &content, focus)))
    }

    fn handle_mutating(
        &self,
        path: &str,
        instruction: &str,
        mode: CommandMode,
    ) -> Result<AgentOutcome> {
        let content = self.executor.file_ops().read(path)?;
        let prompt = match mode {
            CommandMode::Edit => edit_prompt(path, &content, instruction),
            CommandMode::Fix => fix_prompt(path, &content, instruction),
            CommandMode::Refactor => refactor_prompt(path, &content, instruction),
        };
        Ok(AgentOutcome::RunAgentLoop(prompt))
    }

    fn handle_plugin(&self, name: &str, command: &str, args: &[String]) -> Result<AgentOutcome> {
        if !self.plugin_config.enabled {
            return Ok(AgentOutcome::Error(
                "Plugins are disabled in config.".to_string(),
            ));
        }

        let mut manager = PluginManager::new(PluginManager::default_dir());
        manager.load_all()?;
        let output = manager.execute(name, command, args)?;
        Ok(AgentOutcome::Message(output))
    }
}

enum CommandMode {
    Edit,
    Fix,
    Refactor,
}
