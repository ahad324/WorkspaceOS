#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use workspace_engine::{WorkspaceRegistry, WorkspaceMetadata};

struct AppState {
    registry: WorkspaceRegistry,
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
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_active_workspace(
    state: tauri::State<'_, AppState>,
) -> Result<Option<WorkspaceMetadata>, String> {
    Ok(state.registry.get_active_workspace().map(|w| w.metadata))
}

fn main() {
    let registry = WorkspaceRegistry::new();
    let state = AppState { registry };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_runtime_status,
            get_workspaces,
            register_workspace,
            activate_workspace,
            get_active_workspace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
