use serde::{Deserialize, Serialize};
use terminalos_shared::{Error, Result};

/// Configuration for the embedding provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub base_url: String,
    pub model: String,
    pub api_key_env: String,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "nomic-embed-text".to_string(),
            api_key_env: "OLLAMA_API_KEY".to_string(),
        }
    }
}

/// HTTP client for Ollama embedding generation.
pub struct EmbeddingClient {
    config: EmbeddingConfig,
    client: reqwest::Client,
}

impl EmbeddingClient {
    #[must_use]
    pub fn new(config: EmbeddingConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!(
            "{}/api/embeddings",
            self.config.base_url.trim_end_matches('/')
        );
        let body = serde_json::json!({
            "model": self.config.model,
            "prompt": text,
        });

        let mut request = self.client.post(url).json(&body);
        if let Ok(key) = std::env::var(&self.config.api_key_env) {
            if !key.is_empty() {
                request = request.bearer_auth(key);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::Search(format!("embedding request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Search(format!(
                "embedding API error ({status}): {body}"
            )));
        }

        let payload: OllamaEmbeddingResponse = response
            .json()
            .await
            .map_err(|e| Error::Search(format!("embedding decode failed: {e}")))?;

        if payload.embedding.is_empty() {
            return Err(Error::Search(
                "embedding API returned empty vector".to_string(),
            ));
        }

        Ok(payload.embedding)
    }

    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut vectors = Vec::with_capacity(texts.len());
        for text in texts {
            vectors.push(self.embed(text).await?);
        }
        Ok(vectors)
    }

    #[must_use]
    pub fn config(&self) -> &EmbeddingConfig {
        &self.config
    }
}

#[derive(Debug, Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

/// Cosine similarity between two equal-length vectors.
#[must_use]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!(cosine_similarity(&a, &b).abs() < f32::EPSILON);
    }
}
