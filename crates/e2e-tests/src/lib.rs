#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;
    use workspace_engine::{WorkspaceRegistry, WorkspaceState};
    use search_engine::SearchManager;
    use index_engine::WorkspaceIndexer;
    use security_engine::{enforce_path_containment, log_audit_event};
    use mcp_runtime::McpServer;
    use tunnel_manager::{TunnelManager, TunnelState};

    fn setup_temp_workspace() -> PathBuf {
        let id = Uuid::new_v4().to_string();
        let path = std::env::temp_dir().join(format!("workspaceos_e2e_{}", id));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[tokio::test]
    async fn test_full_workspaceos_lifecycle_e2e() {
        let ws_root = setup_temp_workspace();
        let registry_path = ws_root.join("registry.json");

        // 1. Initialize Registry and Register Workspace
        let registry = WorkspaceRegistry::new_with_path(registry_path.clone());
        let ws = registry
            .register_workspace("E2E Test Workspace".to_string(), ws_root.clone())
            .unwrap();

        assert_eq!(ws.metadata.name, "E2E Test Workspace");
        assert_eq!(ws.get_state(), WorkspaceState::Ready);

        // 2. Activate Workspace
        registry.activate_workspace(&ws.metadata.id).unwrap();
        assert_eq!(
            registry.get_active_workspace().unwrap().get_state(),
            WorkspaceState::Active
        );

        // 3. Write mock source files to workspace
        let src_dir = ws_root.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        
        let main_rs = src_dir.join("main.rs");
        fs::write(&main_rs, "fn main() {\n    println!(\"Hello World!\");\n}").unwrap();

        let helper_rs = src_dir.join("helper.rs");
        fs::write(&helper_rs, "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}").unwrap();

        let readme = ws_root.join("README.md");
        fs::write(&readme, "# WorkspaceOS E2E\nThis is an integration test readme.").unwrap();

        // 4. Initialize Indexer and SearchManager
        let event_bus = registry.event_bus.clone();
        let indexer = WorkspaceIndexer::new(&ws, event_bus.clone()).unwrap();
        let searcher = SearchManager::new(&ws, event_bus).unwrap();

        // Perform Initial Indexing
        indexer.perform_initial_index(&ws).unwrap();
        searcher.index_entire_workspace_fts().unwrap();

        // Verify Search Operations
        let path_results = searcher.search_paths("helper").unwrap();
        assert!(!path_results.is_empty());
        assert!(path_results[0].contains("helper.rs"));

        let fts_results = searcher.search_code("integration test").unwrap();
        assert!(!fts_results.is_empty());
        assert!(fts_results[0].path.contains("README.md"));

        // 5. Verify Security Engine Containment
        let safe_path = ws_root.join("src").join("main.rs");
        assert!(enforce_path_containment(&ws_root, &safe_path).is_ok());

        let unsafe_path = ws_root.join("..").join("outside.txt");
        assert!(enforce_path_containment(&ws_root, &unsafe_path).is_err());

        // Log audit event to log file
        log_audit_event(&ws_root, "filesystem.read", "SUCCESS", "e2e validation").unwrap();
        let audit_log_file = ws_root.join(".workspaceos").join("audit.log");
        assert!(audit_log_file.exists());
        let audit_content = fs::read_to_string(&audit_log_file).unwrap();
        assert!(audit_content.contains("filesystem.read"));

        // 6. Verify Tunnel Manager
        let tunnel = TunnelManager::new("Cloudflare");
        let status_init = tunnel.get_status();
        assert_eq!(status_init.state, TunnelState::Disconnected);

        let url = tunnel.start_tunnel().unwrap();
        assert!(url.contains("cloudflare.workspaceos.dev"));
        assert_eq!(tunnel.get_status().state, TunnelState::Connected);

        tunnel.stop_tunnel().unwrap();
        assert_eq!(tunnel.get_status().state, TunnelState::Disconnected);

        // 7. Verify MCP standard commands via server instance
        let server = McpServer::new(ws_root.clone());
        let tools = server.list_tools();
        assert!(tools.iter().any(|t| t.name == "list_dir"));
        assert!(tools.iter().any(|t| t.name == "get_context"));

        // Clean up
        if let Some(watcher) = registry.active_watcher.lock().take() {
            watcher.stop();
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = fs::remove_dir_all(&ws_root);
    }
}
