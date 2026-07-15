//! Workspace Engine
//!
//! Responsible for workspace lifecycle, registration, validation, state, and isolation boundaries.

use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub root: PathBuf,
}

impl Workspace {
    pub fn new(id: String, name: String, root: PathBuf) -> Self {
        Self { id, name, root }
    }

    pub fn validate_path(&self, path: &Path) -> bool {
        path.starts_with(&self.root)
    }
}
