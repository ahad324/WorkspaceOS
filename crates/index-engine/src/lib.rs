//! Index Engine
//!
//! Handles incremental indexing, metadata parsing using Tree-sitter, and persistence using SQLite.

use std::path::Path;
use blake3::Hasher;

pub struct Indexer;

impl Indexer {
    pub fn compute_hash(path: &Path) -> std::io::Result<String> {
        let content = std::fs::read(path)?;
        let mut hasher = Hasher::new();
        hasher.update(&content);
        Ok(hasher.finalize().to_hex().to_string())
    }
}
