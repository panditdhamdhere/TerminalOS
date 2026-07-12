use std::collections::HashMap;

use terminalos_config::{ProviderConfig, ProviderType};
use terminalos_shared::Result;

use crate::provider::AiProvider;

mod anthropic;
mod gemini;
mod openai_compatible;

use anthropic::AnthropicProvider;
use gemini::GeminiProvider;
use openai_compatible::OpenAiCompatibleProvider;

/// Registry of configured AI providers.
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn AiProvider>>,
    default_name: Option<String>,
}

impl ProviderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_name: None,
        }
    }

    pub fn register(&mut self, provider: Box<dyn AiProvider>) {
        if self.default_name.is_none() {
            self.default_name = Some(provider.name().to_string());
        }
        self.providers.insert(provider.name().to_string(), provider);
    }

    pub fn from_configs(configs: &[ProviderConfig], default: Option<&str>) -> Result<Self> {
        let mut registry = Self::new();
        for config in configs {
            if !config.enabled {
                continue;
            }
            let provider = build_provider(config)?;
            registry.register(provider);
        }
        if let Some(name) = default {
            registry.default_name = Some(name.to_string());
        }
        Ok(registry)
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn AiProvider> {
        self.providers.get(name).map(std::convert::AsRef::as_ref)
    }

    #[must_use]
    pub fn default_provider(&self) -> Option<&dyn AiProvider> {
        self.default_name.as_ref().and_then(|name| self.get(name))
    }

    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn build_provider(config: &ProviderConfig) -> Result<Box<dyn AiProvider>> {
    match config.provider_type {
        ProviderType::OpenAi => Ok(Box::new(OpenAiCompatibleProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::Anthropic => Ok(Box::new(AnthropicProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::OpenRouter => Ok(Box::new(OpenAiCompatibleProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::Ollama => Ok(Box::new(OpenAiCompatibleProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::Groq => Ok(Box::new(OpenAiCompatibleProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::Gemini => Ok(Box::new(GeminiProvider::new(
            config.name.clone(),
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string()),
            config.api_key_env.clone(),
            config.model.clone(),
        ))),
        ProviderType::DeepSeek => Ok(Box::new(OpenAiCompatibleProvider::new(
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
