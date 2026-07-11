use futures_util::StreamExt;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use terminalos_shared::{Error, Result};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::message::{ChatMessage, MessageRole};
use crate::provider::{CompletionRequest, CompletionStream, StreamChunk};
use crate::sse::parse_anthropic_chunk;

/// Anthropic Messages API streaming provider.
pub struct AnthropicProvider {
    name: String,
    base_url: String,
    api_key_env: String,
    default_model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
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

impl crate::provider::AiProvider for AnthropicProvider {
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
            if let Err(e) = run_anthropic_stream(
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

async fn run_anthropic_stream(
    client: &reqwest::Client,
    base_url: &str,
    model: &str,
    api_key_env: &str,
    request: &CompletionRequest,
    tx: tokio::sync::mpsc::UnboundedSender<Result<StreamChunk>>,
) -> Result<()> {
    let url = format!("{base_url}/messages");
    let api_key = std::env::var(api_key_env)
        .map_err(|_| Error::Ai(format!("missing API key env var: {api_key_env}")))?;

    let (system, messages) = split_system_messages(&request.messages);

    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "stream": true,
        "temperature": request.temperature,
    });

    if let Some(sys) = system {
        body["system"] = serde_json::Value::String(sys);
    }

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(&api_key).map_err(|e| Error::Ai(e.to_string()))?,
    );
    headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

    let response = client
        .post(&url)
        .headers(headers)
        .json(&body)
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

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].to_string();
            buffer = buffer[pos + 1..].to_string();
            if let Some(content) = parse_anthropic_chunk(&line) {
                tx.send(Ok(StreamChunk {
                    content,
                    done: false,
                }))
                .map_err(|e| Error::Ai(format!("channel closed: {e}")))?;
            }
        }
    }

    Ok(())
}

fn split_system_messages(messages: &[ChatMessage]) -> (Option<String>, Vec<serde_json::Value>) {
    let mut system = None;
    let mut result = Vec::new();

    for msg in messages {
        match msg.role {
            MessageRole::System => {
                system = Some(msg.content.clone());
            }
            MessageRole::User => {
                result.push(serde_json::json!({
                    "role": "user",
                    "content": msg.content,
                }));
            }
            MessageRole::Assistant => {
                result.push(serde_json::json!({
                    "role": "assistant",
                    "content": msg.content,
                }));
            }
        }
    }

    (system, result)
}
