//! Coding agent with slash commands, tool execution, and confirmation gates.

pub mod action;
pub mod command;
pub mod git_assistant;
pub mod prompts;
pub mod session;
pub mod tools;

pub use action::{ActionKind, PendingAction};
pub use command::{SlashCommand, parse_slash_command};
pub use git_assistant::{GitAssistant, extract_commit_message};
pub use session::{AgentOutcome, AgentSession, ConfirmedResult};
