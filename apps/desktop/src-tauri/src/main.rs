#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use index_engine::WorkspaceIndexer;
use search_engine::SearchManager;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use workspace_engine::{WorkspaceMetadata, WorkspaceRegistry};

struct AppState {
    registry: WorkspaceRegistry,
    indexers: Mutex<HashMap<String, Arc<WorkspaceIndexer>>>,
    searchers: Mutex<HashMap<String, Arc<SearchManager>>>,
    tunnel_mgr: tunnel_manager::TunnelManager,
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
fn unregister_workspace(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .registry
        .unregister_workspace(&id)
        .map_err(|e| e.to_string())?;

    state.indexers.lock().unwrap().remove(&id);
    state.searchers.lock().unwrap().remove(&id);
    Ok(())
}

#[tauri::command]
fn activate_workspace(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .registry
        .activate_workspace(&id)
        .map_err(|e| e.to_string())?;

    let ws = state
        .registry
        .get_active_workspaces()
        .into_iter()
        .find(|w| w.metadata.id == id)
        .ok_or_else(|| "Workspace activated but could not be loaded".to_string())?;

    let event_bus = state.registry.event_bus.clone();

    let indexer = Arc::new(WorkspaceIndexer::new(&ws, event_bus.clone())?);
    let searcher = Arc::new(SearchManager::new(&ws, event_bus)?);

    // Run initial indexing in background
    let indexer_bg = indexer.clone();
    let searcher_bg = searcher.clone();
    let ws_bg = ws.clone();
    tokio::spawn(async move {
        let _ = indexer_bg.perform_initial_index(&ws_bg);
        let _ = searcher_bg.index_entire_workspace_fts();
    });

    indexer.clone().start_incremental_listener();
    searcher.clone().start_incremental_listener();

    state.indexers.lock().unwrap().insert(id.clone(), indexer);
    state.searchers.lock().unwrap().insert(id, searcher);

    Ok(())
}

#[tauri::command]
fn deactivate_workspace(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .registry
        .deactivate_workspace(&id)
        .map_err(|e| e.to_string())?;

    state.indexers.lock().unwrap().remove(&id);
    state.searchers.lock().unwrap().remove(&id);

    Ok(())
}

#[tauri::command]
fn get_active_workspace(
    state: tauri::State<'_, AppState>,
) -> Result<Option<WorkspaceMetadata>, String> {
    Ok(state.registry.get_active_workspace().map(|w| w.metadata))
}

#[tauri::command]
fn get_active_workspaces(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<WorkspaceMetadata>, String> {
    Ok(state
        .registry
        .get_active_workspaces()
        .into_iter()
        .map(|w| w.metadata)
        .collect())
}

#[tauri::command]
fn search_paths(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<index_engine::FileRecord>, String> {
    let mut all_results = Vec::new();
    let searchers = state.searchers.lock().unwrap();
    for searcher in searchers.values() {
        if let Ok(res) = searcher.search_paths(&query) {
            all_results.extend(res);
        }
    }
    Ok(all_results)
}

#[tauri::command]
fn search_symbols(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<index_engine::SymbolRecord>, String> {
    let mut all_results = Vec::new();
    let searchers = state.searchers.lock().unwrap();
    for searcher in searchers.values() {
        if let Ok(res) = searcher.search_symbols(&query) {
            all_results.extend(res);
        }
    }
    Ok(all_results)
}

#[tauri::command]
fn search_code(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<search_engine::CodeSearchResult>, String> {
    let mut all_results = Vec::new();
    let searchers = state.searchers.lock().unwrap();
    for searcher in searchers.values() {
        if let Ok(res) = searcher.search_code(&query) {
            all_results.extend(res);
        }
    }
    Ok(all_results)
}

#[tauri::command]
fn generate_context(
    state: tauri::State<'_, AppState>,
    query: String,
    token_budget: Option<usize>,
) -> Result<context_engine::ContextProfile, String> {
    let searchers = state.searchers.lock().unwrap();
    let searcher = searchers
        .values()
        .next()
        .ok_or("No active workspace found")?;
    let active_workspaces = state.registry.get_active_workspaces();
    let ws = active_workspaces
        .first()
        .ok_or("No active workspace loaded")?;
    context_engine::ContextEngine::assemble_context(
        ws,
        searcher,
        &query,
        token_budget.unwrap_or(2000),
    )
}

#[tauri::command]
fn start_tunnel(
    state: tauri::State<'_, AppState>,
    provider: String,
    auth_token: String,
) -> Result<String, String> {
    state.tunnel_mgr.set_provider(&provider);
    println!(
        "Connecting tunnel provider: {} (auth token configured: {})",
        provider,
        !auth_token.is_empty()
    );
    state.tunnel_mgr.start_tunnel()
}

#[tauri::command]
fn stop_tunnel(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.tunnel_mgr.stop_tunnel()
}

#[tauri::command]
fn list_plugins(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<plugin_system::PluginMetadata>, String> {
    if let Some(ws) = state.registry.get_active_workspace() {
        Ok(plugin_system::PluginHost::list_plugins(&ws))
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
fn get_diagnostics() -> Result<workspace_engine::PerformanceDiagnostics, String> {
    Ok(workspace_engine::PerformanceDiagnostics::collect_diagnostics())
}

#[tauri::command]
fn get_workspace_config(
    state: tauri::State<'_, AppState>,
) -> Result<workspace_engine::WorkspaceConfig, String> {
    let active = state
        .registry
        .get_active_workspace()
        .ok_or_else(|| "No active workspace loaded".to_string())?;
    let config = active.read_config().map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
fn update_workspace_config(
    state: tauri::State<'_, AppState>,
    config: workspace_engine::WorkspaceConfig,
) -> Result<(), String> {
    let active = state
        .registry
        .get_active_workspace()
        .ok_or_else(|| "No active workspace loaded".to_string())?;
    active.write_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
async fn pick_directory() -> Result<Option<String>, String> {
    use rfd::AsyncFileDialog;
    let file_handle = AsyncFileDialog::new().pick_folder().await;
    Ok(file_handle.map(|f| f.path().to_string_lossy().into_owned()))
}

#[tauri::command]
fn get_audit_logs(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    use std::fs::read_to_string;
    let active = state.registry.get_active_workspace();
    if let Some(ws) = active {
        let audit_log = ws.metadata.root.join(".workspaceos").join("audit.log");
        if !audit_log.exists() {
            return Ok(vec![
                "[INFO] Audit log file is empty or does not exist yet.".to_string(),
            ]);
        }
        let content = read_to_string(audit_log).map_err(|e| e.to_string())?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Ok(lines)
    } else {
        Ok(vec![
            "[INFO] Audit log file is empty or does not exist yet.".to_string(),
        ])
    }
}

#[tauri::command]
fn clear_audit_logs(state: tauri::State<'_, AppState>) -> Result<(), String> {
    use std::fs::write;
    let active = state.registry.get_active_workspace();
    if let Some(ws) = active {
        let audit_log = ws.metadata.root.join(".workspaceos").join("audit.log");
        if audit_log.exists() {
            write(audit_log, "").map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn main() {
    let registry = WorkspaceRegistry::new();
    let indexers = Mutex::new(HashMap::new());
    let searchers = Mutex::new(HashMap::new());
    let tunnel_mgr = tunnel_manager::TunnelManager::new("Cloudflare");

    let state = AppState {
        registry,
        indexers,
        searchers,
        tunnel_mgr,
    };

    // If there are active workspaces on startup, boot up their indexers/searchers
    let active_workspaces = state.registry.get_active_workspaces();
    for ws in active_workspaces {
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

                state
                    .indexers
                    .lock()
                    .unwrap()
                    .insert(ws.metadata.id.clone(), idx);
                state
                    .searchers
                    .lock()
                    .unwrap()
                    .insert(ws.metadata.id.clone(), sch);
            }
        }
    }

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_runtime_status,
            get_workspaces,
            register_workspace,
            unregister_workspace,
            activate_workspace,
            deactivate_workspace,
            get_active_workspace,
            get_active_workspaces,
            search_paths,
            search_symbols,
            search_code,
            generate_context,
            start_tunnel,
            stop_tunnel,
            list_plugins,
            get_diagnostics,
            get_workspace_config,
            update_workspace_config,
            pick_directory,
            get_audit_logs,
            clear_audit_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
