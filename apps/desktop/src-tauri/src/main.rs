#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn get_runtime_status() -> String {
    "WorkspaceOS Core Online".to_string()
}

fn main() {
    // Basic verification of engine linkages
    let _ws = workspace_engine::Workspace::new(
        "workspace-0".to_string(), 
        "WorkspaceOS".to_string(), 
        std::path::PathBuf::from(".")
    );
    let _eval = security_engine::SecurityEvaluator;

    println!("WorkspaceOS core engine modules loaded successfully.");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_runtime_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
