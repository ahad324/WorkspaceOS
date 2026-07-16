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

#[tauri::command]
fn generate_context(
    state: tauri::State<'_, AppState>,
    query: String,
    token_budget: Option<usize>,
) -> Result<context_engine::ContextProfile, String> {
    let searcher_opt = state.searcher.lock().unwrap();
    let searcher = searcher_opt
        .as_ref()
        .ok_or("No active search manager found")?;
    let ws = state
        .registry
        .get_active_workspace()
        .ok_or("No active workspace loaded")?;
    context_engine::ContextEngine::assemble_context(
        &ws,
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
    let ws = state
        .registry
        .get_active_workspace()
        .ok_or("No active workspace loaded")?;
    Ok(plugin_system::PluginHost::list_plugins(&ws))
}

#[tauri::command]
fn get_diagnostics() -> Result<workspace_engine::PerformanceDiagnostics, String> {
    Ok(workspace_engine::PerformanceDiagnostics::collect_diagnostics())
}

#[tauri::command]
fn get_workspace_config(
    state: tauri::State<'_, AppState>,
) -> Result<workspace_engine::WorkspaceConfig, String> {
    let ws = state
        .registry
        .get_active_workspace()
        .ok_or("No active workspace loaded")?;
    let config_file = ws.metadata.root.join(".workspaceos.toml");
    if config_file.exists() {
        let content = std::fs::read_to_string(&config_file).map_err(|e| e.to_string())?;
        let config: workspace_engine::WorkspaceConfig =
            toml::from_str(&content).map_err(|e| e.to_string())?;
        Ok(config)
    } else {
        Ok(ws.config)
    }
}

#[tauri::command]
fn update_workspace_config(
    state: tauri::State<'_, AppState>,
    config: workspace_engine::WorkspaceConfig,
) -> Result<(), String> {
    let ws = state
        .registry
        .get_active_workspace()
        .ok_or("No active workspace loaded")?;
    let config_file = ws.metadata.root.join(".workspaceos.toml");
    let content = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&config_file, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn pick_directory() -> Result<Option<String>, String> {
    let dir = rfd::FileDialog::new().pick_folder();
    Ok(dir.map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
fn get_audit_logs(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let ws = state
        .registry
        .get_active_workspace()
        .ok_or("No active workspace loaded")?;
    let log_path = ws.metadata.root.join(".workspaceos").join("audit.log");
    if log_path.exists() {
        let content = std::fs::read_to_string(&log_path).map_err(|e| e.to_string())?;
        let lines: Vec<String> = content
            .lines()
            .rev()
            .take(100)
            .map(|s| s.to_string())
            .collect();
        Ok(lines)
    } else {
        Ok(vec![
            "[INFO] Audit log file is empty or does not exist yet.".to_string(),
        ])
    }
}

fn main() {
    let registry = WorkspaceRegistry::new();
    let indexer = Mutex::new(None);
    let searcher = Mutex::new(None);
    let tunnel_mgr = tunnel_manager::TunnelManager::new("Cloudflare");

    let state = AppState {
        registry,
        indexer,
        searcher,
        tunnel_mgr,
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
            search_code,
            generate_context,
            start_tunnel,
            stop_tunnel,
            list_plugins,
            get_diagnostics,
            get_workspace_config,
            update_workspace_config,
            pick_directory,
            get_audit_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
