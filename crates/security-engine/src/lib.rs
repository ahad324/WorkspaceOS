//! Security Engine
//!
//! Enforces authentication, authorization, and capability evaluation before execution.

use serde::{Deserialize, Serialize};
use workspace_engine::Workspace;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    #[serde(rename = "filesystem.read")]
    FilesystemRead,
    #[serde(rename = "filesystem.write")]
    FilesystemWrite,
    #[serde(rename = "filesystem.delete")]
    FilesystemDelete,
    #[serde(rename = "terminal.execute")]
    TerminalExecute,
    #[serde(rename = "git.read")]
    GitRead,
    #[serde(rename = "git.write")]
    GitWrite,
}

pub struct SecurityEvaluator;

impl SecurityEvaluator {
    pub fn authorize(
        _workspace: &Workspace,
        capability: &Capability,
        required_caps: &[Capability],
    ) -> bool {
        required_caps.contains(capability)
    }
}
