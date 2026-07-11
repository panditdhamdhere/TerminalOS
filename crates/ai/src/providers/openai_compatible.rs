use futures_util::StreamExt;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use terminalos_shared::{Error, Result};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::message::{ChatMessage, MessageRole};
use crate::provider::{CompletionRequest, CompletionStream, StreamChunk};
use crate::sse::parse_openai_chunk;

/// OpenAI-compatible streaming provider (OpenAI, OpenRouter, Ollama, DeepSeek).
pub struct OpenAiCompatibleProvider {
    name: String,
    base_url: String,
    api_key_env: String,
    default_model: String,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(name: String, base_url: String, api_key_env: String, default_model: String) -> Self {
        Self {
            name,
            base_url,
            api_key_env,
            default_model,
            client: reqwest::Client::new(),
        }
    }
}

impl crate::provider::AiProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn complete(
        &self,
        request: CompletionRequest,
        handle: tokio::runtime::Handle,
    ) -> CompletionStream {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let api_key_env = self.api_key_env.clone();
        let model = if request.model.is_empty() {
            self.default_model.clone()
        } else {
            request.model.clone()
        };

        handle.spawn(async move {
            if let Err(e) = run_openai_stream(
                &client,
                &base_url,
                &model,
                &api_key_env,
                &request,
                tx.clone(),
            )
            .await
            {
                let _ = tx.send(Err(e));
            }
            let _ = tx.send(Ok(StreamChunk {
                content: String::new(),
                done: true,
            }));
        });

        Box::pin(UnboundedReceiverStream::new(rx))
    }
}

async fn run_openai_stream(
    client: &reqwest::Client,
    base_url: &str,
    model: &str,
    api_key_env: &str,
    request: &CompletionRequest,
    tx: tokio::sync::mpsc::UnboundedSender<Result<StreamChunk>>,
) -> Result<()> {
    let url = format!("{base_url}/chat/completions");
    let api_key = std::env::var(api_key_env).ok();

    let body = serde_json::json!({
        "model": model,
        "messages": request.messages.iter().map(message_to_json).collect::<Vec<_>>(),
        "temperature": request.temperature,
        "stream": true,
        "max_tokens": request.max_tokens,
    });

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let mut req = client.post(&url).headers(headers).json(&body);
    if let Some(key) = api_key.filter(|k| !k.is_empty()) {
        req = req.bearer_auth(key);
    }

    let response = req
        .send()
        .await
        .map_err(|e| Error::Ai(format!("request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(Error::Ai(format!("provider error {status}: {text}")));
    }

    let mut byte_stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = byte_stream.next().await {
        let chunk = chunk.map_err(|e| Error::Ai(format!("stream error: {e}")))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find("\n\n") {
            let event = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();
            for line in event.lines() {
                if let Some(content) = parse_openai_chunk(line) {
                    tx.send(Ok(StreamChunk {
                        content,
                        done: false,
                    }))
                    .map_err(|e| Error::Ai(format!("channel closed: {e}")))?;
                }
            }
        }
    }

    Ok(())
}

fn message_to_json(message: &ChatMessage) -> serde_json::Value {
    serde_json::json!({
        "role": match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        },
        "content": message.content,
    })
}
