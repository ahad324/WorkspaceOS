use crate::fts::TantivyIndex;
use index_engine::{FileRecord, IndexDb, SymbolRecord};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{error, info, warn};
use workspace_engine::{FsEvent, Workspace, WorkspaceEvent, WorkspaceEventBus};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeSearchResult {
    pub path: String,
    pub line: usize,
    pub content: String,
}

pub struct SearchManager {
    workspace_id: String,
    root: PathBuf,
    db: Arc<IndexDb>,
    fts: Arc<TantivyIndex>,
    event_bus: Arc<WorkspaceEventBus>,
}

impl SearchManager {
    pub fn new(ws: &Workspace, event_bus: Arc<WorkspaceEventBus>) -> Result<Self, String> {
        let db_path = ws.metadata.root.join(".workspaceos").join("index.db");
        let db = Arc::new(IndexDb::new(&db_path).map_err(|e| e.to_string())?);

        let fts_path = ws.metadata.root.join(".workspaceos").join("search_index");
        let fts = Arc::new(TantivyIndex::new(&fts_path)?);

        Ok(Self {
            workspace_id: ws.metadata.id.clone(),
            root: ws.metadata.root.clone(),
            db,
            fts,
            event_bus,
        })
    }

    pub fn search_paths(&self, query: &str) -> Result<Vec<FileRecord>, String> {
        // Query files matching the path substring fuzzy search
        let files = self.db.list_files().map_err(|e| e.to_string())?;
        let query_lower = query.to_lowercase();

        let filtered = files
            .into_iter()
            .filter(|f| f.path.to_lowercase().contains(&query_lower))
            .collect();
        Ok(filtered)
    }

    pub fn search_symbols(&self, query: &str) -> Result<Vec<SymbolRecord>, String> {
        let files = self.db.list_files().map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for file in files {
            if let Ok(syms) = self.db.get_symbols_for_file(file.id) {
                for sym in syms {
                    if sym.name.to_lowercase().contains(&query_lower) {
                        results.push(sym);
                    }
                }
            }
        }
        Ok(results)
    }

    pub fn search_code(&self, query: &str) -> Result<Vec<CodeSearchResult>, String> {
        let raw_results = self.fts.search(query)?;
        let mapped = raw_results
            .into_iter()
            .map(|(path, line, content)| CodeSearchResult {
                path,
                line,
                content,
            })
            .collect();
        Ok(mapped)
    }

    pub fn index_file_in_fts(&self, relative_path: &str) -> Result<(), String> {
        let full_path = self.root.join(relative_path);
        if !full_path.exists() || full_path.is_dir() {
            return Ok(());
        }

        if let Ok(body) = fs::read_to_string(&full_path) {
            self.fts.index_document(relative_path, &body)?;
        }
        Ok(())
    }

    pub fn delete_file_from_fts(&self, relative_path: &str) -> Result<(), String> {
        self.fts.delete_document(relative_path)?;
        Ok(())
    }

    pub fn start_incremental_listener(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let mut rx = self.event_bus.subscribe();
        let searcher = Arc::clone(&self);

        tokio::spawn(async move {
            info!(
                "Incremental Search Engine FTS listener started for workspace {}",
                searcher.workspace_id
            );
            while let Ok(event) = rx.recv().await {
                if let WorkspaceEvent::FsUpdate {
                    id,
                    event: fs_event,
                } = event
                {
                    if id != searcher.workspace_id {
                        continue;
                    }

                    match fs_event {
                        FsEvent::Created(ref path) | FsEvent::Modified(ref path) => {
                            let rel_path = path.to_string_lossy();

                            // Prevent indexing the internal sqlite/fts index files themselves!
                            if rel_path.starts_with(".workspaceos") || rel_path.starts_with(".git")
                            {
                                continue;
                            }

                            info!("FTS indexing file change event on: {}", rel_path);
                            if let Err(e) = searcher.index_file_in_fts(&rel_path) {
                                error!(
                                    "Failed to incrementally FTS index file {}: {}",
                                    rel_path, e
                                );
                            }
                        }
                        FsEvent::Deleted(ref path) => {
                            let rel_path = path.to_string_lossy();
                            if rel_path.starts_with(".workspaceos") || rel_path.starts_with(".git")
                            {
                                continue;
                            }
                            info!("Removing FTS record on delete event: {}", rel_path);
                            if let Err(e) = searcher.delete_file_from_fts(&rel_path) {
                                error!(
                                    "Failed to remove FTS index for deleted file {}: {}",
                                    rel_path, e
                                );
                            }
                        }
                        FsEvent::Renamed(ref from, ref to) => {
                            let from_rel = from.to_string_lossy();
                            let to_rel = to.to_string_lossy();
                            if to_rel.starts_with(".workspaceos") || to_rel.starts_with(".git") {
                                continue;
                            }
                            info!("Renaming FTS indexed file from {} to {}", from_rel, to_rel);
                            let _ = searcher.delete_file_from_fts(&from_rel);
                            if let Err(e) = searcher.index_file_in_fts(&to_rel) {
                                error!("Failed to index renamed file in FTS {}: {}", to_rel, e);
                            }
                        }
                    }
                }
            }
        })
    }

    pub fn index_entire_workspace_fts(&self) -> Result<(), String> {
        info!("Indexing entire workspace FTS for {}", self.workspace_id);
        self.index_dir_fts(&self.root)?;
        info!(
            "Completed entire workspace FTS index for {}",
            self.workspace_id
        );
        Ok(())
    }

    fn index_dir_fts(&self, dir: &Path) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(rel_path) = path.strip_prefix(&self.root) {
                let rel_str = rel_path.to_string_lossy();

                if rel_str == ".git"
                    || rel_str == "node_modules"
                    || rel_str == "target"
                    || rel_str == ".workspaceos"
                {
                    continue;
                }

                if path.is_dir() {
                    if let Err(e) = self.index_dir_fts(&path) {
                        warn!("Skipped subdirectory FTS {:?}: {}", path, e);
                    }
                } else if path.is_file() {
                    if let Err(e) = self.index_file_in_fts(&rel_str) {
                        warn!("Failed to FTS index file {:?}: {}", path, e);
                    }
                }
            }
        }
        Ok(())
    }
}
