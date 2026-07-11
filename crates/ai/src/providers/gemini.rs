use futures_util::StreamExt;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use terminalos_shared::{Error, Result};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::message::MessageRole;
use crate::provider::{CompletionRequest, CompletionStream, StreamChunk};
use crate::sse::parse_gemini_chunk;

/// Google Gemini streaming provider.
pub struct GeminiProvider {
    name: String,
    base_url: String,
    api_key_env: String,
    default_model: String,
    client: reqwest::Client,
}

impl GeminiProvider {
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

impl crate::provider::AiProvider for GeminiProvider {
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
            if let Err(e) = run_gemini_stream(
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

async fn run_gemini_stream(
    client: &reqwest::Client,
    base_url: &str,
    model: &str,
    api_key_env: &str,
    request: &CompletionRequest,
    tx: tokio::sync::mpsc::UnboundedSender<Result<StreamChunk>>,
) -> Result<()> {
    let api_key = std::env::var(api_key_env)
        .map_err(|_| Error::Ai(format!("missing API key env var: {api_key_env}")))?;

    let url = format!("{base_url}/models/{model}:streamGenerateContent?alt=sse&key={api_key}");

    let contents: Vec<serde_json::Value> = request
        .messages
        .iter()
        .filter(|m| m.role != MessageRole::System)
        .map(|m| {
            serde_json::json!({
                "role": match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "model",
                    MessageRole::System => "user",
                },
                "parts": [{ "text": m.content }],
            })
        })
        .collect();

    let system_instruction = request
        .messages
        .iter()
        .find(|m| m.role == MessageRole::System)
        .map(|m| serde_json::json!({ "parts": [{ "text": m.content }] }));

    let mut body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "temperature": request.temperature,
            "maxOutputTokens": request.max_tokens.unwrap_or(4096),
        }
    });

    if let Some(sys) = system_instruction {
        body["systemInstruction"] = sys;
    }

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

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
    let mut last_text = String::new();

    while let Some(chunk) = byte_stream.next().await {
        let chunk = chunk.map_err(|e| Error::Ai(format!("stream error: {e}")))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].to_string();
            buffer = buffer[pos + 1..].to_string();

            let data = line.strip_prefix("data:").map(str::trim);
            if let Some(data) = data {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(full) = parse_gemini_chunk(&json) {
                        let delta = if full.starts_with(&last_text) {
                            full[last_text.len()..].to_string()
                        } else {
                            full.clone()
                        };
                        last_text = full;
                        if !delta.is_empty() {
                            tx.send(Ok(StreamChunk {
                                content: delta,
                                done: false,
                            }))
                            .map_err(|e| Error::Ai(format!("channel closed: {e}")))?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
