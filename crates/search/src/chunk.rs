use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A semantically meaningful slice of source code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeChunk {
    pub id: String,
    pub path: String,
    pub symbol: Option<String>,
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
    pub content: String,
}

impl CodeChunk {
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        symbol: Option<String>,
        kind: impl Into<String>,
        start_line: u32,
        end_line: u32,
        content: impl Into<String>,
    ) -> Self {
        let path = path.into();
        let content = content.into();
        let id = chunk_id(&path, start_line, symbol.as_deref().unwrap_or(""));
        Self {
            id,
            path,
            symbol,
            kind: kind.into(),
            start_line,
            end_line,
            content,
        }
    }
}

/// Stable chunk id from path, line, and optional symbol name.
#[must_use]
pub fn chunk_id(path: &str, start_line: u32, symbol: &str) -> String {
    let key = format!("terminalos:chunk:{path}:{start_line}:{symbol}");
    Uuid::new_v5(&Uuid::NAMESPACE_URL, key.as_bytes()).to_string()
}
