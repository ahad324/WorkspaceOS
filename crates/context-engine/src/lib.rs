use search_engine::SearchManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use workspace_engine::Workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    BugFix,
    Feature,
    Refactor,
    ExplainCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnippet {
    pub path: String,
    pub content: String,
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextProfile {
    pub intent: Intent,
    pub snippets: Vec<ContextSnippet>,
    pub total_tokens: usize,
}

pub struct ContextEngine;

impl ContextEngine {
    pub fn detect_intent(query: &str) -> Intent {
        let q = query.to_lowercase();
        if q.contains("fix")
            || q.contains("bug")
            || q.contains("error")
            || q.contains("fail")
            || q.contains("issue")
            || q.contains("crash")
        {
            Intent::BugFix
        } else if q.contains("add")
            || q.contains("implement")
            || q.contains("create")
            || q.contains("new")
            || q.contains("feature")
        {
            Intent::Feature
        } else if q.contains("refactor")
            || q.contains("cleanup")
            || q.contains("clean")
            || q.contains("structure")
        {
            Intent::Refactor
        } else {
            Intent::ExplainCode
        }
    }

    pub fn assemble_context(
        ws: &Workspace,
        search_mgr: &SearchManager,
        query: &str,
        token_budget: usize,
    ) -> Result<ContextProfile, String> {
        let intent = Self::detect_intent(query);

        // 1. Structural + Symbol Search
        let path_matches = search_mgr.search_paths(query).unwrap_or_default();
        let symbol_matches = search_mgr.search_symbols(query).unwrap_or_default();
        let code_matches = search_mgr.search_code(query).unwrap_or_default();

        let mut scores: HashMap<String, (f32, String)> = HashMap::new();

        // Score based on path matches
        for file in path_matches {
            scores.insert(file.path.clone(), (50.0, "Direct path match".to_string()));
        }

        // Score based on symbol matches
        for sym in symbol_matches {
            // Find path of the file containing this symbol
            // In index-engine SQLite database, we can resolve the file record by id
            // Let's get the file list to lookup
            let db_path = ws.metadata.root.join(".workspaceos").join("index.db");
            if let Ok(db) = index_engine::IndexDb::new(&db_path) {
                let files = db.list_files().unwrap_or_default();
                if let Some(f) = files.into_iter().find(|file| file.id == sym.file_id) {
                    let entry = scores.entry(f.path).or_insert((0.0, "".to_string()));
                    entry.0 += 30.0;
                    if entry.1.is_empty() {
                        entry.1 = format!("Contains matching symbol '{}'", sym.name);
                    }
                }
            }
        }

        // Score based on code FTS occurrences
        for match_item in code_matches {
            let entry = scores
                .entry(match_item.path)
                .or_insert((0.0, "".to_string()));
            entry.0 += 20.0;
            if entry.1.is_empty() {
                entry.1 = format!(
                    "Contains matching text occurrence: '{}'",
                    match_item.content
                );
            }
        }

        // Proximity / dependency check
        // If file A imports file B, and A has a score > 0, B gets a boost
        let db_path = ws.metadata.root.join(".workspaceos").join("index.db");
        if let Ok(db) = index_engine::IndexDb::new(&db_path) {
            let files = db.list_files().unwrap_or_default();
            for f in &files {
                if let Some(score_info) = scores.get(&f.path).cloned() {
                    if score_info.0 > 0.0 {
                        // Find dependencies of this file
                        if let Ok(deps) = db.get_dependencies_for_file(f.id) {
                            for (target_id, _kind) in deps {
                                if let Some(target_file) =
                                    files.iter().find(|tf| tf.id == target_id)
                                {
                                    let target_entry = scores
                                        .entry(target_file.path.clone())
                                        .or_insert((0.0, "".to_string()));
                                    target_entry.0 += 15.0;
                                    if target_entry.1.is_empty() {
                                        target_entry.1 =
                                            format!("Imported by relevant file '{}'", f.path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort files by score descending
        let mut scored_files: Vec<(String, f32, String)> = scores
            .into_iter()
            .map(|(path, (score, reason))| (path, score, reason))
            .collect();
        scored_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut snippets = Vec::new();
        let mut total_tokens = 0;

        for (path_str, score, reason) in scored_files {
            let full_path = ws.metadata.root.join(&path_str);
            if full_path.exists() && full_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&full_path) {
                    // Simple word-based token estimation
                    let words = content.split_whitespace().count();
                    let estimated_tokens = (words as f32 * 1.3) as usize;

                    if total_tokens + estimated_tokens <= token_budget {
                        total_tokens += estimated_tokens;
                        snippets.push(ContextSnippet {
                            path: path_str,
                            content,
                            score,
                            reason,
                        });
                    } else if snippets.is_empty() {
                        // If a single file exceeds budget but list is empty, take a truncated version of the file!
                        let lines: Vec<&str> = content.lines().take(100).collect();
                        let truncated = lines.join("\n");
                        let trunc_words = truncated.split_whitespace().count();
                        total_tokens += (trunc_words as f32 * 1.3) as usize;
                        snippets.push(ContextSnippet {
                            path: path_str,
                            content: truncated + "\n... (truncated)",
                            score,
                            reason: format!("{} (Truncated to fit budget)", reason),
                        });
                        break;
                    } else {
                        // Exceeded budget, stop packing
                        break;
                    }
                }
            }
        }

        Ok(ContextProfile {
            intent,
            snippets,
            total_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn setup_temp_workspace() -> PathBuf {
        let path = std::env::temp_dir().join(format!("context_engine_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    #[test]
    fn test_intent_detection() {
        assert_eq!(
            ContextEngine::detect_intent("fix the memory leak"),
            Intent::BugFix
        );
        assert_eq!(
            ContextEngine::detect_intent("implement active sessions"),
            Intent::Feature
        );
        assert_eq!(
            ContextEngine::detect_intent("refactor the registry module"),
            Intent::Refactor
        );
        assert_eq!(
            ContextEngine::detect_intent("explain how state machine works"),
            Intent::ExplainCode
        );
    }

    #[test]
    fn test_context_relevance_ranking() {
        let root = setup_temp_workspace();
        let ws = Workspace::new(
            "ws-id".to_string(),
            "Test Workspace".to_string(),
            root.clone(),
        );
        let event_bus = std::sync::Arc::new(workspace_engine::WorkspaceEventBus::new());

        let index_dir = root.join(".workspaceos");
        std::fs::create_dir_all(&index_dir).unwrap();

        // Setup SQLite database records
        let db_path = index_dir.join("index.db");
        let db = index_engine::IndexDb::new(&db_path).unwrap();

        let main_file = root.join("main.rs");
        std::fs::write(&main_file, "fn main() { println!(\"server starting\"); }").unwrap();

        let file_id = db
            .insert_file("main.rs", 100, 12345, "hash123", "rust")
            .unwrap();
        db.insert_symbol(&index_engine::SymbolRecord {
            id: 0,
            file_id,
            name: "main".to_string(),
            kind: "function".to_string(),
            start_line: 1,
            start_column: 0,
            end_line: 1,
            end_column: 15,
        })
        .unwrap();

        // Create Search Manager
        let search_mgr = SearchManager::new(&ws, event_bus).unwrap();

        // Assemble context
        let profile = ContextEngine::assemble_context(&ws, &search_mgr, "main", 1000).unwrap();
        assert_eq!(profile.snippets.len(), 1);
        assert_eq!(profile.snippets[0].path, "main.rs");
        assert!(profile.snippets[0].score > 0.0);

        let _ = std::fs::remove_dir_all(&root);
    }
}
