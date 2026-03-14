//! Search module for rustchat
//!
//! Provides OpenSearch integration with fallback to PostgreSQL FTS.

pub mod client;
pub mod indexer;
pub mod management;

// Re-export OpenSearchConfig from config module
pub use crate::config::OpenSearchConfig;
pub use client::OpenSearchClient;
pub use indexer::{PostDocument, SearchIndexer};
pub use management::IndexManager;

/// Initialize the search module
pub fn init() {
    tracing::info!("Search module initialized");
}
