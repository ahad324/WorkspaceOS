//! Search Engine
//!
//! Provides hybrid search using SQLite for metadata and Tantivy for full-text search.

pub struct SearchEngine;

impl SearchEngine {
    pub fn search(query: &str) -> Vec<String> {
        tracing::info!("Searching for: {}", query);
        vec![]
    }
}
