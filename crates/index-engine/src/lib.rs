pub mod db;
pub mod indexer;
pub mod parser;

pub use db::{FileRecord, IndexDb, SymbolRecord};
pub use indexer::WorkspaceIndexer;
pub use parser::{extract_symbols, ParsedSymbol};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn setup_temp_workspace() -> PathBuf {
        let path = std::env::temp_dir().join(format!("index_engine_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    #[test]
    fn test_db_operations() {
        let root = setup_temp_workspace();
        let db_path = root.join("index.db");
        let db = IndexDb::new(&db_path).unwrap();

        // Test file insertion
        let file_id = db
            .insert_file("src/lib.rs", 100, 12345, "hash123", "rust")
            .unwrap();
        assert!(file_id > 0);

        // Test get file
        let file = db.get_file("src/lib.rs").unwrap().unwrap();
        assert_eq!(file.size, 100);
        assert_eq!(file.hash, "hash123");
        assert_eq!(file.language, "rust");

        // Test symbol insertion
        let sym = SymbolRecord {
            id: 0,
            file_id,
            name: "test_function".to_string(),
            kind: "function".to_string(),
            start_line: 10,
            start_column: 4,
            end_line: 12,
            end_column: 5,
        };
        let sym_id = db.insert_symbol(&sym).unwrap();
        assert!(sym_id > 0);

        // Test get symbols
        let symbols = db.get_symbols_for_file(file_id).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "test_function");
        assert_eq!(symbols[0].kind, "function");

        // Test cascade deletion
        db.delete_file("src/lib.rs").unwrap();
        let file_post_delete = db.get_file("src/lib.rs").unwrap();
        assert!(file_post_delete.is_none());

        let symbols_post_delete = db.get_symbols_for_file(file_id).unwrap();
        assert_eq!(symbols_post_delete.len(), 0);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_symbol_extraction() {
        // Rust Test
        let rust_code = r#"
            pub struct User {
                name: String,
            }
            fn get_user_name() -> String {
                "name".to_string()
            }
        "#;
        let rust_symbols = extract_symbols("rust", rust_code);
        assert_eq!(rust_symbols.len(), 2);

        let struct_sym = rust_symbols.iter().find(|s| s.kind == "struct").unwrap();
        assert_eq!(struct_sym.name, "User");

        let fn_sym = rust_symbols.iter().find(|s| s.kind == "function").unwrap();
        assert_eq!(fn_sym.name, "get_user_name");

        // TypeScript Test
        let ts_code = r#"
            interface Employee {
                id: number;
            }
            class Manager implements Employee {
                id = 1;
            }
        "#;
        let ts_symbols = extract_symbols("typescript", ts_code);
        assert_eq!(ts_symbols.len(), 2);

        let interface_sym = ts_symbols.iter().find(|s| s.kind == "interface").unwrap();
        assert_eq!(interface_sym.name, "Employee");

        let class_sym = ts_symbols.iter().find(|s| s.kind == "class").unwrap();
        assert_eq!(class_sym.name, "Manager");
    }

    #[test]
    fn test_indexer_flow() {
        let root = setup_temp_workspace();

        // Setup mock Workspace and event bus
        let ws = workspace_engine::Workspace::new(
            "ws-id".to_string(),
            "Test Workspace".to_string(),
            root.clone(),
        );
        let event_bus = std::sync::Arc::new(workspace_engine::WorkspaceEventBus::new());

        let indexer = WorkspaceIndexer::new(&ws, event_bus).unwrap();

        // Write a test rust file
        let src_dir = root.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let test_file = src_dir.join("lib.rs");
        std::fs::write(&test_file, "struct Context; fn run() {}").unwrap();

        // Index it
        indexer.index_file("src/lib.rs").unwrap();

        // Query database to check if symbols appear
        let file_rec = indexer.db.get_file("src/lib.rs").unwrap().unwrap();
        assert_eq!(file_rec.language, "rust");

        let symbols = indexer.db.get_symbols_for_file(file_rec.id).unwrap();
        assert_eq!(symbols.len(), 2);
        assert!(symbols
            .iter()
            .any(|s| s.name == "Context" && s.kind == "struct"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "run" && s.kind == "function"));

        // Test incremental modification
        std::fs::write(&test_file, "struct Context; fn run() {} fn stop() {}").unwrap();
        indexer.index_file("src/lib.rs").unwrap();

        let symbols_mod = indexer.db.get_symbols_for_file(file_rec.id).unwrap();
        assert_eq!(symbols_mod.len(), 3);
        assert!(symbols_mod
            .iter()
            .any(|s| s.name == "stop" && s.kind == "function"));

        // Clean up
        let _ = std::fs::remove_dir_all(&root);
    }
}
