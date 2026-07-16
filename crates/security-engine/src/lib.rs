//! Security Engine
//!
//! Enforces authentication, authorization, and capability evaluation before execution.

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
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

    pub fn validate_token(token: &str) -> bool {
        // Authenticate against optional WORKSPACEOS_API_KEY environment variable.
        // If not set, we generate a stable local session validation rule.
        if let Ok(expected) = std::env::var("WORKSPACEOS_API_KEY") {
            token == expected
        } else {
            // Local dev default fallback (if no env variable is set, allow non-empty tokens)
            !token.is_empty()
        }
    }

    pub fn enforce_path_containment(ws: &Workspace, path: &Path) -> Result<PathBuf, String> {
        let canonical_root = ws
            .metadata
            .root
            .canonicalize()
            .unwrap_or_else(|_| ws.metadata.root.clone());

        // Early reject absolute or drive/root-relative path escapes
        let has_root = path.components().any(|c| matches!(c, std::path::Component::RootDir | std::path::Component::Prefix(_)));
        if has_root && !path.starts_with(&ws.metadata.root) && !path.starts_with(&canonical_root) {
            return Err(format!(
                "Security Violation: Root-relative or absolute path outside workspace containment boundary: {:?}",
                path
            ));
        }

        // 1. Resolve full path (if relative, join with workspace root)
        let full_path = if path.is_relative() {
            ws.metadata.root.join(path)
        } else {
            path.to_path_buf()
        };

        // 2. Canonicalize path if it exists to resolve symlinks and traversals
        let canonical_path = match full_path.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // If it doesn't exist yet (e.g. creating a new file), we normalize components
                let mut normalized = ws.metadata.root.clone();
                for component in path.components() {
                    match component {
                        std::path::Component::Normal(c) => normalized.push(c),
                        std::path::Component::ParentDir => {
                            normalized.pop();
                        }
                        std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                            // Skip absolute roots to prevent escaping
                        }
                        _ => {}
                    }
                }
                normalized
            }
        };

        // 3. Verify that canonical path resides within workspace root boundary
        let canonical_root = ws
            .metadata
            .root
            .canonicalize()
            .unwrap_or_else(|_| ws.metadata.root.clone());

        if canonical_path.starts_with(&canonical_root) {
            Ok(canonical_path)
        } else {
            Err(format!(
                "Security Violation: Path traversal escape detected on {:?}",
                path
            ))
        }
    }

    pub fn audit_log(ws: &Workspace, action: &str, event: &str, success: bool) {
        let audit_dir = ws.metadata.root.join(".workspaceos");
        std::fs::create_dir_all(&audit_dir).unwrap_or_default();
        let log_path = audit_dir.join("audit.log");

        let status = if success { "SUCCESS" } else { "DENIED" };
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let log_line = format!(
            "[{}] {} - Action: {}, Details: {}\n",
            timestamp, status, action, event
        );

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
            let _ = file.write_all(log_line.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn setup_temp_workspace() -> PathBuf {
        let path = std::env::temp_dir().join(format!("security_engine_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    #[test]
    fn test_path_containment() {
        let root = setup_temp_workspace();
        let ws = Workspace::new(
            "ws-id".to_string(),
            "Test Workspace".to_string(),
            root.clone(),
        );

        // Safe relative path
        let safe_rel = Path::new("src/lib.rs");
        let res = SecurityEvaluator::enforce_path_containment(&ws, safe_rel);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), root.join("src/lib.rs"));

        // Unsafe escape path
        let unsafe_escape = Path::new("../escaped_dir/secret.txt");
        let res_escape = SecurityEvaluator::enforce_path_containment(&ws, unsafe_escape);
        assert!(res_escape.is_err());

        // Unsafe absolute path
        let absolute_escape = Path::new("/etc/passwd");
        let res_abs = SecurityEvaluator::enforce_path_containment(&ws, absolute_escape);
        assert!(res_abs.is_err());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_audit_logging() {
        let root = setup_temp_workspace();
        let ws = Workspace::new(
            "ws-id".to_string(),
            "Test Workspace".to_string(),
            root.clone(),
        );

        SecurityEvaluator::audit_log(&ws, "filesystem.read", "view_file src/lib.rs", true);

        let log_file = root.join(".workspaceos").join("audit.log");
        assert!(log_file.exists());
        let content = std::fs::read_to_string(&log_file).unwrap();
        assert!(content.contains("SUCCESS"));
        assert!(content.contains("filesystem.read"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
