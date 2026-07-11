use std::pin::Pin;

use futures_util::StreamExt;
use terminalos_config::AppConfig;
use terminalos_shared::{Error, Result};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::message::{ChatMessage, MessageRole};
use crate::provider::{CompletionRequest, StreamChunk};
use crate::providers::ProviderRegistry;

/// UI-facing chat message with streaming state.
#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub role: MessageRole,
    pub content: String,
    pub streaming: bool,
}

/// Manages AI chat conversations with streaming provider responses.
pub struct ChatEngine {
    registry: ProviderRegistry,
    messages: Vec<DisplayMessage>,
    provider_name: String,
    model: String,
    stream_rx: Option<UnboundedReceiver<Result<StreamChunk>>>,
    streaming_index: Option<usize>,
}

impl ChatEngine {
    pub fn from_config(config: &AppConfig) -> Result<Self> {
        let registry =
            ProviderRegistry::from_configs(&config.providers, config.default_provider.as_deref())?;

        let provider_name = config
            .default_provider
            .clone()
            .or_else(|| registry.names().first().cloned())
            .unwrap_or_else(|| "none".to_string());

        let model = config
            .providers
            .iter()
            .find(|p| p.name == provider_name)
            .map(|p| p.model.clone())
            .unwrap_or_default();

        Ok(Self {
            registry,
            messages: vec![DisplayMessage {
                role: MessageRole::System,
                content: "You are TerminalOS AI, a helpful coding assistant.".to_string(),
                streaming: false,
            }],
            provider_name,
            model,
            stream_rx: None,
            streaming_index: None,
        })
    }

    #[must_use]
    pub fn messages(&self) -> &[DisplayMessage] {
        &self.messages
    }

    #[must_use]
    pub fn provider_name(&self) -> &str {
        &self.provider_name
    }

    #[must_use]
    pub fn is_streaming(&self) -> bool {
        self.streaming_index.is_some()
    }

    #[must_use]
    pub fn has_providers(&self) -> bool {
        !self.registry.is_empty()
    }

    pub fn submit(&mut self, content: String) -> Result<()> {
        if content.trim().is_empty() {
            return Ok(());
        }
        if self.is_streaming() {
            return Err(Error::Ai("already streaming a response".to_string()));
        }

        self.messages.push(DisplayMessage {
            role: MessageRole::User,
            content,
            streaming: false,
        });

        let provider = self
            .registry
            .get(&self.provider_name)
            .or_else(|| self.registry.default_provider())
            .ok_or_else(|| {
                Error::Ai(
                    "no AI provider configured — add one to ~/.config/terminalos/config.toml"
                        .to_string(),
                )
            })?;

        let request_messages: Vec<ChatMessage> = self
            .messages
            .iter()
            .map(|m| ChatMessage {
                role: m.role,
                content: m.content.clone(),
            })
            .collect();

        let request = CompletionRequest {
            messages: request_messages,
            model: self.model.clone(),
            temperature: 0.7,
            max_tokens: Some(4096),
        };

        let mut stream: Pin<Box<dyn futures_util::Stream<Item = Result<StreamChunk>> + Send>> =
            provider.complete(request);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            while let Some(chunk) = stream.next().await {
                if tx.send(chunk).is_err() {
                    break;
                }
            }
        });

        self.messages.push(DisplayMessage {
            role: MessageRole::Assistant,
            content: String::new(),
            streaming: true,
        });
        self.streaming_index = Some(self.messages.len() - 1);
        self.stream_rx = Some(rx);

        Ok(())
    }

    pub fn poll_stream(&mut self) -> bool {
        let Some(mut rx) = self.stream_rx.take() else {
            return false;
        };

        let mut updated = false;
        let mut finished = false;

        while let Ok(chunk) = rx.try_recv() {
            updated = true;
            match chunk {
                Ok(stream_chunk) => {
                    if let Some(idx) = self.streaming_index {
                        if stream_chunk.done {
                            self.messages[idx].streaming = false;
                            finished = true;
                        } else {
                            self.messages[idx].content.push_str(&stream_chunk.content);
                        }
                    }
                }
                Err(e) => {
                    if let Some(idx) = self.streaming_index {
                        self.messages[idx].content = format!("Error: {e}");
                        self.messages[idx].streaming = false;
                    }
                    finished = true;
                }
            }
        }

        if finished {
            self.streaming_index = None;
        } else {
            self.stream_rx = Some(rx);
        }

        updated
    }

    pub fn load_history(&mut self, records: Vec<(MessageRole, String)>) {
        if records.is_empty() {
            return;
        }
        self.messages.retain(|m| m.role == MessageRole::System);
        for (role, content) in records {
            self.messages.push(DisplayMessage {
                role,
                content,
                streaming: false,
            });
        }
    }

    #[must_use]
    pub fn history_for_save(&self) -> Vec<(MessageRole, String)> {
        self.messages
            .iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|m| (m.role, m.content.clone()))
            .collect()
    }

    /// Appends a display message without triggering a provider call.
    pub fn push_message(&mut self, role: MessageRole, content: String) {
        self.messages.push(DisplayMessage {
            role,
            content,
            streaming: false,
        });
    }

    /// Runs a one-shot completion for agent tool loops (non-streaming).
    pub fn complete_sync(
        &self,
        messages: &[ChatMessage],
        runtime: &tokio::runtime::Runtime,
    ) -> Result<String> {
        let provider = self
            .registry
            .get(&self.provider_name)
            .or_else(|| self.registry.default_provider())
            .ok_or_else(|| Error::Ai("no AI provider configured".to_string()))?;

        let request = CompletionRequest {
            messages: messages.to_vec(),
            model: self.model.clone(),
            temperature: 0.2,
            max_tokens: Some(8192),
        };

        let mut stream = provider.complete(request);
        runtime.block_on(async {
            let mut content = String::new();
            while let Some(chunk) = stream.next().await {
                match chunk? {
                    StreamChunk {
                        content: part,
                        done,
                    } if !done => content.push_str(&part),
                    _ => {}
                }
            }
            Ok(content)
        })
    }
}
