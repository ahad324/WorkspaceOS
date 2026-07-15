pub mod config;
pub mod event_bus;
pub mod ignore;
pub mod state;
pub mod watcher;

use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

pub use config::WorkspaceConfig;
pub use event_bus::{FsEvent, WorkspaceEvent, WorkspaceEventBus};
pub use ignore::IgnoreMatcher;
pub use state::WorkspaceState;
pub use watcher::WorkspaceWatcher;

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("Workspace not found: {0}")]
    NotFound(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Illegal state transition: {0}")]
    IllegalStateTransition(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    IO(#[from] io::Error),

    #[error("Duplicate workspace path: {0}")]
    DuplicateWorkspace(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    pub id: String,
    pub name: String,
    pub root: PathBuf,
    pub created_at: u64,
    pub last_modified: u64,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub metadata: WorkspaceMetadata,
    pub state: Arc<RwLock<WorkspaceState>>,
    pub config: WorkspaceConfig,
}

impl Workspace {
    pub fn new(id: String, name: String, root: PathBuf) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            metadata: WorkspaceMetadata {
                id,
                name,
                root,
                created_at: now,
                last_modified: now,
            },
            state: Arc::new(RwLock::new(WorkspaceState::Created)),
            config: WorkspaceConfig::default(),
        }
    }

    pub fn resolve_path(&self, relative_path: &str) -> Result<PathBuf, WorkspaceError> {
        let root_canonical = self.metadata.root.canonicalize().map_err(|e| {
            WorkspaceError::InvalidPath(format!("Failed to canonicalize workspace root: {}", e))
        })?;

        // Handle absolute path inputs - reject immediately
        let raw_path = Path::new(relative_path);
        if raw_path.is_absolute() {
            return Err(WorkspaceError::InvalidPath(
                "Absolute paths are not allowed".to_string(),
            ));
        }

        let joined_path = self.metadata.root.join(relative_path);

        let path_canonical = match joined_path.canonicalize() {
            Ok(p) => p,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // If it is a new file that does not exist, resolve parent directory
                if let Some(parent) = joined_path.parent() {
                    let parent_canonical = parent.canonicalize().map_err(|err| {
                        WorkspaceError::InvalidPath(format!(
                            "Parent directory does not exist or cannot be canonicalized: {}",
                            err
                        ))
                    })?;
                    if let Some(filename) = joined_path.file_name() {
                        parent_canonical.join(filename)
                    } else {
                        return Err(WorkspaceError::InvalidPath("Invalid file name".to_string()));
                    }
                } else {
                    return Err(WorkspaceError::InvalidPath(format!(
                        "Path cannot be resolved: {}",
                        e
                    )));
                }
            }
            Err(e) => return Err(WorkspaceError::IO(e)),
        };

        if path_canonical.starts_with(&root_canonical) {
            Ok(path_canonical)
        } else {
            Err(WorkspaceError::InvalidPath(format!(
                "Path containment violation: {:?} escapes workspace root {:?}",
                path_canonical, root_canonical
            )))
        }
    }

    pub fn set_state(&self, new_state: WorkspaceState) -> Result<(), WorkspaceError> {
        let mut state_guard = self.state.write();
        let updated_state = state_guard
            .transition_to(new_state)
            .map_err(WorkspaceError::IllegalStateTransition)?;
        *state_guard = updated_state;
        Ok(())
    }

    pub fn get_state(&self) -> WorkspaceState {
        *self.state.read()
    }
}

// Global registry schema for persistence
#[derive(Debug, Serialize, Deserialize, Default)]
struct RegistryData {
    workspaces: Vec<WorkspaceMetadata>,
    active_id: Option<String>,
}

pub struct WorkspaceRegistry {
    registry_path: PathBuf,
    workspaces: RwLock<Vec<Workspace>>,
    active_id: RwLock<Option<String>>,
    pub event_bus: Arc<WorkspaceEventBus>,
    active_watcher: Arc<Mutex<Option<WorkspaceWatcher>>>,
}

impl Default for WorkspaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRegistry {
    pub fn new() -> Self {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());

        let registry_path = PathBuf::from(home)
            .join(".workspaceos")
            .join("registry.json");

        let reg = Self {
            registry_path,
            workspaces: RwLock::new(Vec::new()),
            active_id: RwLock::new(None),
            event_bus: Arc::new(WorkspaceEventBus::new()),
            active_watcher: Arc::new(Mutex::new(None)),
        };

        if let Err(e) = reg.load_registry() {
            warn!(
                "Could not load workspace registry: {}. Initializing empty registry.",
                e
            );
        }

        reg
    }

    pub fn load_registry(&self) -> Result<(), WorkspaceError> {
        if !self.registry_path.exists() {
            return Ok(());
        }

        let file = File::open(&self.registry_path)?;
        let data: RegistryData = serde_json::from_reader(file).map_err(|e| {
            WorkspaceError::ConfigError(format!("Failed to parse registry JSON: {}", e))
        })?;

        let mut workspaces_guard = self.workspaces.write();
        let mut active_id_guard = self.active_id.write();

        workspaces_guard.clear();
        for meta in data.workspaces {
            let ws = Workspace {
                metadata: meta,
                state: Arc::new(RwLock::new(WorkspaceState::Ready)),
                config: WorkspaceConfig::default(),
            };
            workspaces_guard.push(ws);
        }

        *active_id_guard = data.active_id;
        Ok(())
    }

    pub fn save_registry(&self) -> Result<(), WorkspaceError> {
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let workspaces_guard = self.workspaces.read();
        let active_id_guard = self.active_id.read();

        let data = RegistryData {
            workspaces: workspaces_guard
                .iter()
                .map(|w| w.metadata.clone())
                .collect(),
            active_id: active_id_guard.clone(),
        };

        let file = File::create(&self.registry_path)?;
        serde_json::to_writer_pretty(file, &data).map_err(|e| {
            WorkspaceError::ConfigError(format!("Failed to write registry JSON: {}", e))
        })?;

        Ok(())
    }

    pub fn register_workspace(
        &self,
        name: String,
        root: PathBuf,
    ) -> Result<Workspace, WorkspaceError> {
        let root_canonical = root.canonicalize().map_err(|e| {
            WorkspaceError::InvalidPath(format!("Failed to resolve workspace path: {}", e))
        })?;

        let workspaces_guard = self.workspaces.read();
        for ws in workspaces_guard.iter() {
            if ws.metadata.root == root_canonical {
                return Err(WorkspaceError::DuplicateWorkspace(
                    root_canonical.to_string_lossy().into_owned(),
                ));
            }
        }
        drop(workspaces_guard);

        let id = Uuid::new_v4().to_string();
        let ws = Workspace::new(id, name, root_canonical);

        // Write default configuration file (.workspaceos.toml) to workspace root if missing
        let config_file = ws.metadata.root.join(".workspaceos.toml");
        if !config_file.exists() {
            let default_config = toml::to_string_pretty(&ws.config).map_err(|e| {
                WorkspaceError::ConfigError(format!("Failed to serialize default TOML: {}", e))
            })?;
            let mut file = File::create(config_file)?;
            file.write_all(default_config.as_bytes())?;
        }

        let mut workspaces_guard = self.workspaces.write();
        workspaces_guard.push(ws.clone());
        drop(workspaces_guard);

        self.save_registry()?;
        self.event_bus.publish(WorkspaceEvent::WorkspaceCreated {
            id: ws.metadata.id.clone(),
            name: ws.metadata.name.clone(),
            root: ws.metadata.root.clone(),
        });
        info!(
            "Workspace registered successfully: {} at {:?}",
            ws.metadata.name, ws.metadata.root
        );

        Ok(ws)
    }

    pub fn activate_workspace(&self, id: &str) -> Result<(), WorkspaceError> {
        let workspaces_guard = self.workspaces.read();
        let active_ws = workspaces_guard
            .iter()
            .find(|w| w.metadata.id == id)
            .cloned();
        let active_ws = match active_ws {
            Some(w) => w,
            None => return Err(WorkspaceError::NotFound(id.to_string())),
        };

        // Set all workspaces state to Ready, except the active one
        for ws in workspaces_guard.iter() {
            if ws.metadata.id == id {
                ws.set_state(WorkspaceState::Active)?;
            } else if ws.get_state() == WorkspaceState::Active {
                ws.set_state(WorkspaceState::Ready)?;
            }
        }
        drop(workspaces_guard);

        // Stop active watcher if present
        let mut watcher_guard = self.active_watcher.lock();
        if let Some(old_watcher) = watcher_guard.take() {
            old_watcher.stop();
        }

        // Spawn new watcher
        let new_watcher = WorkspaceWatcher::new(
            &active_ws.metadata.id,
            active_ws.metadata.root.clone(),
            Arc::clone(&self.event_bus),
        )?;
        *watcher_guard = Some(new_watcher);
        drop(watcher_guard);

        let mut active_id_guard = self.active_id.write();
        *active_id_guard = Some(id.to_string());
        drop(active_id_guard);

        self.save_registry()?;
        self.event_bus
            .publish(WorkspaceEvent::WorkspaceActivated { id: id.to_string() });
        info!("Workspace activated: {}", id);
        Ok(())
    }

    pub fn get_active_workspace(&self) -> Option<Workspace> {
        let active_id_guard = self.active_id.read();
        let workspaces_guard = self.workspaces.read();

        if let Some(ref id) = *active_id_guard {
            workspaces_guard
                .iter()
                .find(|w| w.metadata.id == *id)
                .cloned()
        } else {
            None
        }
    }

    pub fn list_workspaces(&self) -> Vec<Workspace> {
        self.workspaces.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_temp_workspace() -> PathBuf {
        let path = std::env::temp_dir().join(format!("workspaceos_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn test_state_transitions() {
        let ws = Workspace::new(
            "test-id".to_string(),
            "Test Workspace".to_string(),
            PathBuf::from("."),
        );
        assert_eq!(ws.get_state(), WorkspaceState::Created);

        assert!(ws.set_state(WorkspaceState::Initializing).is_ok());
        assert_eq!(ws.get_state(), WorkspaceState::Initializing);

        assert!(ws.set_state(WorkspaceState::Ready).is_ok());
        assert_eq!(ws.get_state(), WorkspaceState::Ready);

        // Invalid transitions should fail
        assert!(ws.set_state(WorkspaceState::Indexing).is_err());
    }

    #[test]
    fn test_path_resolution_and_containment() {
        let root = setup_temp_workspace();
        let ws = Workspace::new(
            "test-id".to_string(),
            "Test Workspace".to_string(),
            root.clone(),
        );

        // File inside workspace
        let inside = ws.resolve_path("src/lib.rs").unwrap();
        assert!(inside.starts_with(&root));

        // Absolute path input - reject
        assert!(ws.resolve_path("/etc/passwd").is_err());

        // Path traversal attempts
        assert!(ws.resolve_path("../outside.txt").is_err());
        assert!(ws.resolve_path("src/../../outside.txt").is_err());

        // Clean up temp dir
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_ignore_matcher() {
        let root = setup_temp_workspace();

        // Create an ignore matcher
        let matcher = IgnoreMatcher::new(&root);

        // Check hardcoded rules
        assert!(matcher.is_ignored(&root.join(".git").join("config")));
        assert!(matcher.is_ignored(&root.join("node_modules").join("package.json")));
        assert!(matcher.is_ignored(&root.join("target").join("debug").join("binary")));
        assert!(!matcher.is_ignored(&root.join("src").join("lib.rs")));

        // Create a custom gitignore
        let gitignore_path = root.join(".gitignore");
        std::fs::write(&gitignore_path, "*.log\nbuild/").unwrap();

        // Refresh matcher
        let matcher_custom = IgnoreMatcher::new(&root);
        assert!(matcher_custom.is_ignored(&root.join("error.log")));
        assert!(matcher_custom.is_ignored(&root.join("build").join("output.txt")));
        assert!(!matcher_custom.is_ignored(&root.join("src").join("error.log.rs")));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_watcher_events() {
        let root = setup_temp_workspace();

        // We need a test registry with temp path
        let test_registry_path = root.join("registry.json");
        let registry = WorkspaceRegistry {
            registry_path: test_registry_path,
            workspaces: RwLock::new(Vec::new()),
            active_id: RwLock::new(None),
            event_bus: Arc::new(WorkspaceEventBus::new()),
            active_watcher: Arc::new(Mutex::new(None)),
        };

        // Register and activate
        let ws = registry
            .register_workspace("Test".to_string(), root.clone())
            .unwrap();
        registry.activate_workspace(&ws.metadata.id).unwrap();

        // Subscribe to event bus
        let mut rx = registry.event_bus.subscribe();

        // Create a file to trigger the watcher
        let test_file = root.join("test_file.txt");
        std::fs::write(&test_file, "hello").unwrap();

        // Wait up to 2 seconds for the debounced event
        let start = std::time::Instant::now();
        let mut received = false;

        while start.elapsed() < std::time::Duration::from_secs(2) {
            if let Ok(WorkspaceEvent::FsUpdate {
                id,
                event: FsEvent::Created(path),
            }) = rx.try_recv()
            {
                assert_eq!(id, ws.metadata.id);
                assert_eq!(path, Path::new("test_file.txt"));
                received = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        assert!(received, "Did not receive debounced FsUpdate event");

        // Clean up watcher to release locks
        if let Some(watcher) = registry.active_watcher.lock().take() {
            watcher.stop();
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
        let _ = std::fs::remove_dir_all(&root);
    }
}
