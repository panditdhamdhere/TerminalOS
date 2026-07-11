//! Interchangeable AI provider backends with streaming support.

pub mod message;
pub mod provider;
pub mod providers;

pub use message::{ChatMessage, MessageRole};
pub use provider::{AiProvider, CompletionRequest, CompletionStream};
pub use providers::ProviderRegistry;
