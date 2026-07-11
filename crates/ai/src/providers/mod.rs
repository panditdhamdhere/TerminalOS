use std::collections::HashMap;

use futures_util::stream;
use terminalos_config::{ProviderConfig, ProviderType};
use terminalos_shared::{Error, Result};

use crate::provider::{AiProvider, CompletionRequest, CompletionStream, StreamChunk};

/// Registry of configured AI providers.
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn AiProvider>>,
}

impl ProviderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn AiProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    pub fn from_configs(configs: &[ProviderConfig]) -> Result<Self> {
        let mut registry = Self::new();
        for config in configs {
            if !config.enabled {
                continue;
            }
            let provider = build_provider(config)?;
            registry.register(provider);
        }
        Ok(registry)
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn AiProvider> {
        self.providers.get(name).map(std::convert::AsRef::as_ref)
    }

    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn build_provider(config: &ProviderConfig) -> Result<Box<dyn AiProvider>> {
    match config.provider_type {
        ProviderType::OpenAi => Ok(Box::new(HttpProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::Anthropic => Ok(Box::new(HttpProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::OpenRouter => Ok(Box::new(HttpProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::Ollama => Ok(Box::new(HttpProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::Gemini => Ok(Box::new(HttpProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::DeepSeek => Ok(Box::new(HttpProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
    }
}

/// HTTP-based provider that performs streaming chat completions.
struct HttpProvider {
    name: String,
    base_url: String,
    api_key_env: String,
    model: String,
    client: reqwest::Client,
}

impl HttpProvider {
    fn new(name: String, base_url: String, api_key_env: String, model: String) -> Self {
        Self {
            name,
            base_url,
            api_key_env,
            model,
            client: reqwest::Client::new(),
        }
    }
}

impl AiProvider for HttpProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn complete(&self, request: CompletionRequest) -> CompletionStream {
        let api_key = std::env::var(&self.api_key_env).ok();
        let base_url = self.base_url.clone();
        let model = self.model.clone();
        let client = self.client.clone();

        Box::pin(stream::unfold(
            (client, base_url, model, api_key, request, false),
            |(client, base_url, model, api_key, request, started)| async move {
                if started {
                    return None;
                }

                let result =
                    perform_completion(&client, &base_url, &model, api_key.as_deref(), &request)
                        .await;
                Some((result, (client, base_url, model, api_key, request, true)))
            },
        ))
    }
}

async fn perform_completion(
    client: &reqwest::Client,
    base_url: &str,
    model: &str,
    api_key: Option<&str>,
    request: &CompletionRequest,
) -> Result<StreamChunk> {
    let url = format!("{base_url}/chat/completions");

    let body = serde_json::json!({
        "model": model,
        "messages": request.messages.iter().map(|m| {
            serde_json::json!({
                "role": match m.role {
                    crate::message::MessageRole::System => "system",
                    crate::message::MessageRole::User => "user",
                    crate::message::MessageRole::Assistant => "assistant",
                },
                "content": m.content,
            })
        }).collect::<Vec<_>>(),
        "temperature": request.temperature,
        "stream": false,
        "max_tokens": request.max_tokens,
    });

    let mut req = client.post(&url).json(&body);
    if let Some(key) = api_key {
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

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| Error::Ai(format!("invalid response: {e}")))?;

    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(StreamChunk {
        content,
        done: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::ChatMessage;

    #[test]
    fn registry_accepts_provider() {
        let mut registry = ProviderRegistry::new();
        let config = ProviderConfig {
            name: "test".to_string(),
            provider_type: ProviderType::Ollama,
            api_key_env: "OLLAMA_API_KEY".to_string(),
            base_url: None,
            model: "llama3".to_string(),
            enabled: true,
        };
        let provider = build_provider(&config).expect("build");
        registry.register(provider);
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn chat_message_constructors() {
        let msg = ChatMessage::user("hello");
        assert_eq!(msg.content, "hello");
    }
}
