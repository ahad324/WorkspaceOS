use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{info, warn};
use workspace_engine::Workspace;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

pub struct PluginHost;

impl PluginHost {
    pub fn list_plugins(ws: &Workspace) -> Vec<PluginMetadata> {
        let mut plugins = Vec::new();

        // 1. Load built-in default companion plugins
        plugins.push(PluginMetadata {
            name: "git-companion".to_string(),
            version: "1.0.0".to_string(),
            description: "Exposes Git status, commit, and history tools".to_string(),
            capabilities: vec!["git.read".to_string(), "git.write".to_string()],
        });
        plugins.push(PluginMetadata {
            name: "db-inspector".to_string(),
            version: "0.9.0".to_string(),
            description: "Direct SQLite inspection utilities for structured databases".to_string(),
            capabilities: vec!["filesystem.read".to_string()],
        });

        // 2. Scan external workspace plugin directories (.workspaceos/plugins/)
        let plugins_dir = ws.metadata.root.join(".workspaceos").join("plugins");
        if plugins_dir.exists() && plugins_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(plugins_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            match serde_json::from_str::<PluginMetadata>(&content) {
                                Ok(meta) => {
                                    info!("Loaded external plugin manifest: {}", meta.name);
                                    plugins.push(meta);
                                }
                                Err(e) => {
                                    warn!("Failed to parse plugin manifest at {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        plugins
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn setup_temp_workspace() -> PathBuf {
        let path = std::env::temp_dir().join(format!("plugin_system_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    #[test]
    fn test_plugin_host_listing() {
        let root = setup_temp_workspace();
        let ws = Workspace::new(
            "ws-id".to_string(),
            "Test Workspace".to_string(),
            root.clone(),
        );

        // Create external plugin mock manifest file
        let plugins_dir = root.join(".workspaceos").join("plugins");
        std::fs::create_dir_all(&plugins_dir).unwrap();

        let plugin_file = plugins_dir.join("test-plugin.json");
        let manifest_content = r#"{
            "name": "test-plugin",
            "version": "1.2.3",
            "description": "Mock user plugin",
            "capabilities": ["filesystem.read"]
        }"#;
        std::fs::write(&plugin_file, manifest_content).unwrap();

        // Load plugins
        let list = PluginHost::list_plugins(&ws);

        // Verify built-ins and custom loaded plugin
        assert!(list.iter().any(|p| p.name == "git-companion"));
        assert!(list.iter().any(|p| p.name == "db-inspector"));
        assert!(list
            .iter()
            .any(|p| p.name == "test-plugin" && p.version == "1.2.3"));

        let _ = std::fs::remove_dir_all(&root);
    }
}
