use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

use crate::chunk::CodeChunk;

/// Supported source languages for tree-sitter parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Unknown,
}

impl SourceLanguage {
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Self::Rust,
            "py" => Self::Python,
            "js" | "jsx" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "tsx" => Self::TypeScript,
            "go" => Self::Go,
            _ => Self::Unknown,
        }
    }

    fn tree_sitter_language(self) -> Option<Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            Self::Python => Some(tree_sitter_python::LANGUAGE.into()),
            Self::JavaScript | Self::TypeScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            Self::Go => Some(tree_sitter_go::LANGUAGE.into()),
            Self::Unknown => None,
        }
    }

    fn query_source(self) -> Option<&'static str> {
        match self {
            Self::Rust => Some(RUST_QUERY),
            Self::Python => Some(PYTHON_QUERY),
            Self::JavaScript | Self::TypeScript => Some(JAVASCRIPT_QUERY),
            Self::Go => Some(GO_QUERY),
            Self::Unknown => None,
        }
    }
}

const RUST_QUERY: &str = r#"
(function_item name: (identifier) @name) @item
(struct_item name: (type_identifier) @name) @item
(enum_item name: (type_identifier) @name) @item
(trait_item name: (type_identifier) @name) @item
(impl_item) @item
(mod_item name: (identifier) @name) @item
"#;

const PYTHON_QUERY: &str = r#"
(function_definition name: (identifier) @name) @item
(class_definition name: (identifier) @name) @item
"#;

const JAVASCRIPT_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @item
(class_declaration name: (identifier) @name) @item
(method_definition name: (property_identifier) @name) @item
"#;

const GO_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @item
(method_declaration name: (field_identifier) @name) @item
(type_declaration (type_spec name: (type_identifier) @name)) @item
"#;

/// Extracts semantic code chunks from a source file.
pub struct CodeParser {
    parser: Parser,
}

impl Default for CodeParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeParser {
    #[must_use]
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    pub fn extract_chunks(&mut self, path: &str, source: &str) -> Vec<CodeChunk> {
        let ext = path.rsplit('.').next().unwrap_or("");
        let language = SourceLanguage::from_extension(ext);

        if let Some(chunks) = self.extract_with_tree_sitter(path, source, language) {
            if !chunks.is_empty() {
                return chunks;
            }
        }

        fallback_chunks(path, source)
    }

    fn extract_with_tree_sitter(
        &mut self,
        path: &str,
        source: &str,
        language: SourceLanguage,
    ) -> Option<Vec<CodeChunk>> {
        let ts_language = language.tree_sitter_language()?;
        let query_source = language.query_source()?;

        self.parser.set_language(&ts_language).ok()?;
        let tree = self.parser.parse(source, None)?;
        let root = tree.root_node();
        let query = Query::new(&ts_language, query_source).ok()?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, root, source.as_bytes());

        let mut chunks = Vec::new();
        while let Some(query_match) = matches.next() {
            let mut item_node = None;
            let mut symbol = None;

            for capture in query_match.captures {
                let name = query.capture_names()[capture.index as usize];
                let node = capture.node;
                match name {
                    "item" => item_node = Some(node),
                    "name" => {
                        symbol = node.utf8_text(source.as_bytes()).ok().map(str::to_string);
                    }
                    _ => {}
                }
            }

            let Some(node) = item_node else {
                continue;
            };

            let content = node.utf8_text(source.as_bytes()).ok()?.to_string();
            if content.trim().is_empty() {
                continue;
            }

            let start_line = node.start_position().row as u32 + 1;
            let end_line = node.end_position().row as u32 + 1;
            let kind = node.kind().to_string();

            chunks.push(CodeChunk::new(
                path, symbol, kind, start_line, end_line, content,
            ));
        }

        Some(chunks)
    }
}

fn fallback_chunks(path: &str, source: &str) -> Vec<CodeChunk> {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    const WINDOW: usize = 48;
    const STEP: usize = 36;
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < lines.len() {
        let end = (start + WINDOW).min(lines.len());
        let content = lines[start..end].join("\n");
        if !content.trim().is_empty() {
            chunks.push(CodeChunk::new(
                path,
                None,
                "block",
                start as u32 + 1,
                end as u32,
                content,
            ));
        }
        if end == lines.len() {
            break;
        }
        start += STEP;
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rust_function() {
        let source = r#"
fn hello() {
    println!("hi");
}

struct Widget {
    id: u32,
}
"#;
        let mut parser = CodeParser::new();
        let chunks = parser.extract_chunks("lib.rs", source);
        assert!(chunks.iter().any(|c| c.kind == "function_item"));
        assert!(chunks.iter().any(|c| c.kind == "struct_item"));
    }

    #[test]
    fn fallback_for_unknown_extension() {
        let source = "line one\nline two\nline three";
        let mut parser = CodeParser::new();
        let chunks = parser.extract_chunks("notes.txt", source);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].kind, "block");
    }
}
