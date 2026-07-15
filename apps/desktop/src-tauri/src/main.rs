#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use index_engine::WorkspaceIndexer;
use search_engine::SearchManager;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use workspace_engine::{WorkspaceMetadata, WorkspaceRegistry};

struct AppState {
    registry: WorkspaceRegistry,
    indexer: Mutex<Option<Arc<WorkspaceIndexer>>>,
    searcher: Mutex<Option<Arc<SearchManager>>>,
}

#[tauri::command]
fn get_runtime_status() -> String {
    "WorkspaceOS Core Online".to_string()
}

#[tauri::command]
fn get_workspaces(state: tauri::State<'_, AppState>) -> Result<Vec<WorkspaceMetadata>, String> {
    Ok(state
        .registry
        .list_workspaces()
        .into_iter()
        .map(|w| w.metadata)
        .collect())
}

#[tauri::command]
fn register_workspace(
    state: tauri::State<'_, AppState>,
    name: String,
    path: String,
) -> Result<WorkspaceMetadata, String> {
    let root = PathBuf::from(path);
    let ws = state
        .registry
        .register_workspace(name, root)
        .map_err(|e| e.to_string())?;
    Ok(ws.metadata)
}

#[tauri::command]
fn activate_workspace(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .registry
        .activate_workspace(&id)
        .map_err(|e| e.to_string())?;

    let ws = state
        .registry
        .get_active_workspace()
        .ok_or_else(|| "Workspace activated but could not be loaded".to_string())?;

    let event_bus = state.registry.event_bus.clone();

    let indexer = Arc::new(WorkspaceIndexer::new(&ws, event_bus.clone())?);
    let searcher = Arc::new(SearchManager::new(&ws, event_bus)?);

    // Run initial indexing in background so the UI loads instantly
    let indexer_bg = indexer.clone();
    let searcher_bg = searcher.clone();
    let ws_bg = ws.clone();
    tokio::spawn(async move {
        if let Err(e) = indexer_bg.perform_initial_index(&ws_bg) {
            tracing::error!("Failed to index workspace: {}", e);
        }
        if let Err(e) = searcher_bg.index_entire_workspace_fts() {
            tracing::error!("Failed to FTS index workspace: {}", e);
        }
    });

    // Start watches
    indexer.clone().start_incremental_listener();
    searcher.clone().start_incremental_listener();

    *state.indexer.lock().unwrap() = Some(indexer);
    *state.searcher.lock().unwrap() = Some(searcher);

    Ok(())
}

#[tauri::command]
fn get_active_workspace(
    state: tauri::State<'_, AppState>,
) -> Result<Option<WorkspaceMetadata>, String> {
    Ok(state.registry.get_active_workspace().map(|w| w.metadata))
}

#[tauri::command]
fn search_paths(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<index_engine::FileRecord>, String> {
    let searcher_opt = state.searcher.lock().unwrap();
    let searcher = searcher_opt
        .as_ref()
        .ok_or("No active search manager found")?;
    searcher.search_paths(&query)
}

#[tauri::command]
fn search_symbols(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<index_engine::SymbolRecord>, String> {
    let searcher_opt = state.searcher.lock().unwrap();
    let searcher = searcher_opt
        .as_ref()
        .ok_or("No active search manager found")?;
    searcher.search_symbols(&query)
}

#[tauri::command]
fn search_code(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<search_engine::CodeSearchResult>, String> {
    let searcher_opt = state.searcher.lock().unwrap();
    let searcher = searcher_opt
        .as_ref()
        .ok_or("No active search manager found")?;
    searcher.search_code(&query)
}

fn main() {
    let registry = WorkspaceRegistry::new();
    let indexer = Mutex::new(None);
    let searcher = Mutex::new(None);

    let state = AppState {
        registry,
        indexer,
        searcher,
    };

    // If there is already an active workspace on startup, boot up the indexer/searcher
    if let Some(ws) = state.registry.get_active_workspace() {
        let event_bus = state.registry.event_bus.clone();
        if let Ok(idx) = WorkspaceIndexer::new(&ws, event_bus.clone()) {
            if let Ok(sch) = SearchManager::new(&ws, event_bus) {
                let idx = Arc::new(idx);
                let sch = Arc::new(sch);

                let idx_bg = idx.clone();
                let sch_bg = sch.clone();
                let ws_bg = ws.clone();
                tokio::spawn(async move {
                    let _ = idx_bg.perform_initial_index(&ws_bg);
                    let _ = sch_bg.index_entire_workspace_fts();
                });

                idx.clone().start_incremental_listener();
                sch.clone().start_incremental_listener();

                *state.indexer.lock().unwrap() = Some(idx);
                *state.searcher.lock().unwrap() = Some(sch);
            }
        }
    }

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_runtime_status,
            get_workspaces,
            register_workspace,
            activate_workspace,
            get_active_workspace,
            search_paths,
            search_symbols,
            search_code
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
