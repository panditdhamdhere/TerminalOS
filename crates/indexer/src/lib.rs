//! Project indexing with incremental updates.

pub mod indexer;

pub use indexer::{
    IndexStats, ProjectIndexer, hybrid_config_from_embedding, semantic_db_for_index,
};
