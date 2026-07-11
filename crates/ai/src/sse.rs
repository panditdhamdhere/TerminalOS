//! Server-Sent Events parsing utilities.

/// Parses an OpenAI-compatible SSE data line into a content delta.
#[must_use]
pub fn parse_openai_chunk(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("data:") {
        return None;
    }
    let data = line.strip_prefix("data:")?.trim();
    if data == "[DONE]" {
        return None;
    }
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    json.pointer("/choices/0/delta/content")
        .or_else(|| json.pointer("/choices/0/message/content"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// Parses an Anthropic SSE data line into a text delta.
#[must_use]
pub fn parse_anthropic_chunk(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("data:") {
        return None;
    }
    let data = line.strip_prefix("data:")?.trim();
    let json: serde_json::Value = serde_json::from_str(data).ok()?;
    match json["type"].as_str() {
        Some("content_block_delta") => json
            .pointer("/delta/text")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        Some("message_stop") => None,
        _ => None,
    }
}

/// Parses a Gemini streaming JSON chunk.
#[must_use]
pub fn parse_gemini_chunk(json: &serde_json::Value) -> Option<String> {
    json.pointer("/candidates/0/content/parts/0/text")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openai_delta() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hi"}}]}"#;
        assert_eq!(parse_openai_chunk(line), Some("Hi".to_string()));
    }

    #[test]
    fn parses_anthropic_delta() {
        let line = r#"data: {"type":"content_block_delta","delta":{"text":"Hello"}}"#;
        assert_eq!(parse_anthropic_chunk(line), Some("Hello".to_string()));
    }
}
