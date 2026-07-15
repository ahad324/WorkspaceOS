pub mod fts;
pub mod search;

pub use fts::TantivyIndex;
pub use search::{CodeSearchResult, SearchManager};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;
    use workspace_engine::Workspace;

    fn setup_temp_workspace() -> PathBuf {
        let path = std::env::temp_dir().join(format!("search_engine_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    #[test]
    fn test_fts_operations() {
        let root = setup_temp_workspace();
        let fts_dir = root.join("fts_index");
        let fts = TantivyIndex::new(&fts_dir).unwrap();

        // Index mock documents
        fts.index_document("src/lib.rs", "fn run_server() {\n  let x = 42;\n}")
            .unwrap();
        fts.index_document("src/main.rs", "fn main() {\n  println!(\"hello\");\n}")
            .unwrap();

        // Search body
        let results = fts.search("run_server").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "src/lib.rs");
        assert_eq!(results[0].1, 1); // line 1
        assert_eq!(results[0].2, "fn run_server() {");

        // Delete document
        fts.delete_document("src/lib.rs").unwrap();
        let results_post = fts.search("run_server").unwrap();
        assert_eq!(results_post.len(), 0);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_search_manager_paths_and_symbols() {
        let root = setup_temp_workspace();
        let ws = Workspace::new(
            "ws-id".to_string(),
            "Test Workspace".to_string(),
            root.clone(),
        );
        let event_bus = std::sync::Arc::new(workspace_engine::WorkspaceEventBus::new());

        let manager = SearchManager::new(&ws, event_bus).unwrap();

        // Setup some SQLite file records & symbol records
        let index_dir = root.join(".workspaceos");
        std::fs::create_dir_all(&index_dir).unwrap();

        let db_path = index_dir.join("index.db");
        let db = index_engine::IndexDb::new(&db_path).unwrap();
        let file_id = db
            .insert_file("src/main.rs", 123, 456, "hash1", "rust")
            .unwrap();
        db.insert_symbol(&index_engine::SymbolRecord {
            id: 0,
            file_id,
            name: "calculate_sum".to_string(),
            kind: "function".to_string(),
            start_line: 5,
            start_column: 0,
            end_line: 10,
            end_column: 1,
        })
        .unwrap();

        // Fuzzy path matches
        let path_matches = manager.search_paths("main").unwrap();
        assert_eq!(path_matches.len(), 1);
        assert_eq!(path_matches[0].path, "src/main.rs");

        // Symbol matches
        let symbol_matches = manager.search_symbols("calculate").unwrap();
        assert_eq!(symbol_matches.len(), 1);
        assert_eq!(symbol_matches[0].name, "calculate_sum");

        let _ = std::fs::remove_dir_all(&root);
    }
}
