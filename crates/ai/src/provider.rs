use std::pin::Pin;

use futures_util::Stream;
use terminalos_shared::Result;

use crate::message::ChatMessage;

/// Request payload for a chat completion.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<u32>,
}

/// Streaming token chunk from an AI provider.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
}

/// Stream of completion chunks.
pub type CompletionStream = Pin<Box<dyn Stream<Item = Result<StreamChunk>> + Send>>;

/// Trait implemented by all AI provider backends.
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;

    fn complete(&self, request: CompletionRequest) -> CompletionStream;
}
