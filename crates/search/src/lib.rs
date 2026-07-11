//! Keyword and semantic search powered by Tantivy and vector embeddings.

pub mod chunk;
pub mod embeddings;
pub mod engine;
pub mod hybrid;
pub mod parser;
pub mod vector;

pub use chunk::{CodeChunk, chunk_id};
pub use embeddings::{EmbeddingClient, EmbeddingConfig, cosine_similarity};
pub use engine::{SearchEngine, SearchHit, SearchQuery};
pub use hybrid::{HybridSearchConfig, HybridSearchEngine, SearchMode};
pub use parser::CodeParser;
pub use vector::{SemanticHit, VectorStore, embedding_text};
