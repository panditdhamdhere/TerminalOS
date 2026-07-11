//! Interchangeable AI provider backends with streaming support.

pub mod engine;
pub mod message;
pub mod provider;
pub mod providers;
pub mod sse;

pub use engine::{ChatEngine, DisplayMessage};
pub use message::{ChatMessage, MessageRole};
pub use provider::{AiProvider, CompletionRequest, CompletionStream, StreamChunk};
pub use providers::ProviderRegistry;
