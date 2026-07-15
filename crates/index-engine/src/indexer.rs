use crate::db::{IndexDb, SymbolRecord};
use crate::parser::extract_symbols;
use blake3::Hasher;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info, warn};
use workspace_engine::{FsEvent, Workspace, WorkspaceEvent, WorkspaceEventBus};

pub struct WorkspaceIndexer {
    workspace_id: String,
    root: PathBuf,
    pub db: Arc<IndexDb>,
    event_bus: Arc<WorkspaceEventBus>,
}

impl WorkspaceIndexer {
    pub fn new(ws: &Workspace, event_bus: Arc<WorkspaceEventBus>) -> Result<Self, String> {
        let db_path = ws.metadata.root.join(".workspaceos").join("index.db");
        let db = Arc::new(IndexDb::new(&db_path).map_err(|e| e.to_string())?);

        Ok(Self {
            workspace_id: ws.metadata.id.clone(),
            root: ws.metadata.root.clone(),
            db,
            event_bus,
        })
    }

    pub fn compute_hash(&self, full_path: &Path) -> std::io::Result<String> {
        let content = fs::read(full_path)?;
        let mut hasher = Hasher::new();
        hasher.update(&content);
        Ok(hasher.finalize().to_hex().to_string())
    }

    fn detect_language(path: &Path) -> String {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("tsx") => "tsx".to_string(),
            Some("js") | Some("jsx") => "javascript".to_string(),
            Some(ext) => ext.to_string(),
            None => "".to_string(),
        }
    }

    pub fn index_file(&self, relative_path: &str) -> Result<(), String> {
        let full_path = self.root.join(relative_path);
        if !full_path.exists() {
            return Err("File does not exist".to_string());
        }

        let metadata = fs::metadata(&full_path).map_err(|e| e.to_string())?;
        let size = metadata.len();
        let mtime = metadata
            .modified()
            .and_then(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .map_err(std::io::Error::other)
            })
            .unwrap_or(0);

        let hash = self.compute_hash(&full_path).map_err(|e| e.to_string())?;
        let language = Self::detect_language(&full_path);

        // Check if file is already indexed with the same hash
        if let Ok(Some(existing)) = self.db.get_file(relative_path) {
            if existing.hash == hash {
                // Contents are identical, skip indexing to optimize performance!
                return Ok(());
            }
        }

        // Insert or update file record in SQLite
        let file_id = self
            .db
            .insert_file(relative_path, size, mtime, &hash, &language)
            .map_err(|e| e.to_string())?;

        // Clear old symbols for this file (in case of modifying)
        self.db
            .delete_symbols_for_file(file_id)
            .map_err(|e| e.to_string())?;

        // Extract and insert symbols for supported languages
        if language == "rust"
            || language == "typescript"
            || language == "tsx"
            || language == "javascript"
        {
            if let Ok(code) = fs::read_to_string(&full_path) {
                let symbols = extract_symbols(&language, &code);
                for sym in symbols {
                    let record = SymbolRecord {
                        id: 0,
                        file_id,
                        name: sym.name,
                        kind: sym.kind,
                        start_line: sym.start_line,
                        start_column: sym.start_column,
                        end_line: sym.end_line,
                        end_column: sym.end_column,
                    };
                    if let Err(e) = self.db.insert_symbol(&record) {
                        warn!("Failed to insert symbol into SQLite: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn delete_file(&self, relative_path: &str) -> Result<(), String> {
        self.db
            .delete_file(relative_path)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn start_incremental_listener(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let mut rx = self.event_bus.subscribe();
        let indexer = Arc::clone(&self);

        tokio::spawn(async move {
            info!(
                "Incremental Index Engine listener started for workspace {}",
                indexer.workspace_id
            );
            while let Ok(event) = rx.recv().await {
                if let WorkspaceEvent::FsUpdate {
                    id,
                    event: fs_event,
                } = event
                {
                    if id != indexer.workspace_id {
                        continue;
                    }

                    match fs_event {
                        FsEvent::Created(ref path) | FsEvent::Modified(ref path) => {
                            let rel_path = path.to_string_lossy();
                            info!("Indexing file change event on: {}", rel_path);
                            if let Err(e) = indexer.index_file(&rel_path) {
                                error!("Failed to incrementally index file {}: {}", rel_path, e);
                            }
                        }
                        FsEvent::Deleted(ref path) => {
                            let rel_path = path.to_string_lossy();
                            info!("Removing file record on delete event: {}", rel_path);
                            if let Err(e) = indexer.delete_file(&rel_path) {
                                error!(
                                    "Failed to remove index for deleted file {}: {}",
                                    rel_path, e
                                );
                            }
                        }
                        FsEvent::Renamed(ref from, ref to) => {
                            let from_rel = from.to_string_lossy();
                            let to_rel = to.to_string_lossy();
                            info!("Renaming indexed file from {} to {}", from_rel, to_rel);
                            let _ = indexer.delete_file(&from_rel);
                            if let Err(e) = indexer.index_file(&to_rel) {
                                error!("Failed to index renamed file {}: {}", to_rel, e);
                            }
                        }
                    }
                }
            }
        })
    }

    pub fn perform_initial_index(&self, ws: &Workspace) -> Result<(), String> {
        info!("Performing initial index of workspace {}", ws.metadata.name);
        self.index_dir(&ws.metadata.root)?;
        info!("Initial index complete for workspace {}", ws.metadata.name);
        Ok(())
    }

    fn index_dir(&self, dir: &Path) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();

            // Let's resolve containment checks and ignore checks
            // We construct a temporary relative path to run ignore filters
            if let Ok(rel_path) = path.strip_prefix(&self.root) {
                let rel_str = rel_path.to_string_lossy();

                // Simple ignore matching checks to skip traversing .git, target, node_modules folders
                if rel_str == ".git"
                    || rel_str == "node_modules"
                    || rel_str == "target"
                    || rel_str == ".workspaceos"
                {
                    continue;
                }

                if path.is_dir() {
                    if let Err(e) = self.index_dir(&path) {
                        warn!("Skipped subdirectory {:?}: {}", path, e);
                    }
                } else if path.is_file() {
                    if let Err(e) = self.index_file(&rel_str) {
                        warn!("Failed to index file {:?}: {}", path, e);
                    }
                }
            }
        }
        Ok(())
    }
}
