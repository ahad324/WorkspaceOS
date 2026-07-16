use context_engine::ContextEngine;
use search_engine::SearchManager;
use security_engine::{Capability, SecurityEvaluator};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use workspace_engine::{Workspace, WorkspaceEventBus};

pub struct McpServer {
    ws: Workspace,
    search_mgr: Arc<SearchManager>,
}

impl McpServer {
    pub fn new(root_path: PathBuf) -> Result<Self, String> {
        let ws = Workspace::new(
            "mcp-session-ws".to_string(),
            "MCP Workspace".to_string(),
            root_path,
        );
        let event_bus = Arc::new(WorkspaceEventBus::new());
        let search_mgr = Arc::new(SearchManager::new(&ws, event_bus)?);
        Ok(Self { ws, search_mgr })
    }

    pub async fn run(&self) {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin).lines();
        let mut stdout = io::stdout();

        while let Ok(Some(line)) = reader.next_line().await {
            if let Ok(req) = serde_json::from_str::<Value>(&line) {
                if let Some(id) = req.get("id") {
                    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                    let params = req.get("params").cloned().unwrap_or(json!({}));

                    let response = self.handle_request(id.clone(), method, params).await;
                    let resp_str = serde_json::to_string(&response).unwrap_or_default() + "\n";
                    let _ = stdout.write_all(resp_str.as_bytes()).await;
                    let _ = stdout.flush().await;
                }
            }
        }
    }

    pub async fn handle_request(&self, id: Value, method: &str, params: Value) -> Value {
        match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "WorkspaceOS-MCP",
                        "version": "0.1.0"
                    }
                }
            }),

            "initialized" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {}
            }),

            "tools/list" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": [
                        {
                            "name": "list_dir",
                            "description": "List the files and directories inside the active workspace",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": {
                                        "type": "string",
                                        "description": "Relative path to list"
                                    }
                                },
                                "required": ["path"]
                            }
                        },
                        {
                            "name": "view_file",
                            "description": "View the text content of a file in the workspace",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": {
                                        "type": "string",
                                        "description": "Relative path to the target file"
                                    }
                                },
                                "required": ["path"]
                            }
                        },
                        {
                            "name": "write_to_file",
                            "description": "Create or overwrite a file with specific contents",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": {
                                        "type": "string",
                                        "description": "Relative path to write code to"
                                    },
                                    "content": {
                                        "type": "string",
                                        "description": "Exact text contents to save"
                                    }
                                },
                                "required": ["path", "content"]
                            }
                        },
                        {
                            "name": "search_paths",
                            "description": "Perform fuzzy search on file paths in the repository",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "Substring of file path to look for"
                                    }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "search_symbols",
                            "description": "Search code symbol names (classes, methods, structs) inside files",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "Symbol name query"
                                    }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "search_code",
                            "description": "Search occurrences of text inside file bodies",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "Text query string"
                                    }
                                },
                                "required": ["query"]
                            }
                        },
                        {
                            "name": "get_context",
                            "description": "Assemble relevance-ranked context profile tailored to query",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "query": {
                                        "type": "string",
                                        "description": "User intent query context"
                                    },
                                    "token_budget": {
                                        "type": "integer",
                                        "description": "Maximum tokens (default 2000)"
                                    }
                                },
                                "required": ["query"]
                            }
                        }
                    ]
                }
            }),

            "tools/call" => {
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                self.call_tool(id, tool_name, args).await
            }

            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }),
        }
    }

    async fn call_tool(&self, id: Value, name: &str, args: Value) -> Value {
        // Enforce basic permissions
        let permitted_caps = vec![
            Capability::FilesystemRead,
            Capability::FilesystemWrite,
            Capability::FilesystemDelete,
        ];

        match name {
            "list_dir" => {
                if !SecurityEvaluator::authorize(
                    &self.ws,
                    &Capability::FilesystemRead,
                    &permitted_caps,
                ) {
                    return self.make_error(
                        id,
                        -32001,
                        "Permission Denied: read capability not authorized",
                    );
                }

                let raw_path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
                let target = Path::new(raw_path);

                match SecurityEvaluator::enforce_path_containment(&self.ws, target) {
                    Ok(safe_path) => {
                        SecurityEvaluator::audit_log(
                            &self.ws,
                            "filesystem.read",
                            &format!("list_dir {:?}", safe_path),
                            true,
                        );
                        if let Ok(entries) = std::fs::read_dir(safe_path) {
                            let mut names = Vec::new();
                            for entry in entries.flatten() {
                                if let Some(n) = entry.file_name().to_str() {
                                    names.push(n.to_string());
                                }
                            }
                            self.make_success(id, json!({ "content": [{ "type": "text", "text": names.join("\n") }] }))
                        } else {
                            self.make_error(id, -32002, "Failed to read directory")
                        }
                    }
                    Err(e) => {
                        SecurityEvaluator::audit_log(
                            &self.ws,
                            "filesystem.read",
                            &format!("list_dir denied: {}", e),
                            false,
                        );
                        self.make_error(id, -32003, &e)
                    }
                }
            }

            "view_file" => {
                if !SecurityEvaluator::authorize(
                    &self.ws,
                    &Capability::FilesystemRead,
                    &permitted_caps,
                ) {
                    return self.make_error(
                        id,
                        -32001,
                        "Permission Denied: read capability not authorized",
                    );
                }

                let raw_path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
                let target = Path::new(raw_path);

                match SecurityEvaluator::enforce_path_containment(&self.ws, target) {
                    Ok(safe_path) => {
                        SecurityEvaluator::audit_log(
                            &self.ws,
                            "filesystem.read",
                            &format!("view_file {:?}", safe_path),
                            true,
                        );
                        if let Ok(content) = std::fs::read_to_string(safe_path) {
                            self.make_success(
                                id,
                                json!({ "content": [{ "type": "text", "text": content }] }),
                            )
                        } else {
                            self.make_error(id, -32002, "Failed to read file")
                        }
                    }
                    Err(e) => {
                        SecurityEvaluator::audit_log(
                            &self.ws,
                            "filesystem.read",
                            &format!("view_file denied: {}", e),
                            false,
                        );
                        self.make_error(id, -32003, &e)
                    }
                }
            }

            "write_to_file" => {
                if !SecurityEvaluator::authorize(
                    &self.ws,
                    &Capability::FilesystemWrite,
                    &permitted_caps,
                ) {
                    return self.make_error(
                        id,
                        -32001,
                        "Permission Denied: write capability not authorized",
                    );
                }

                let raw_path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
                let content = args.get("content").and_then(|c| c.as_str()).unwrap_or("");
                let target = Path::new(raw_path);

                match SecurityEvaluator::enforce_path_containment(&self.ws, target) {
                    Ok(safe_path) => {
                        SecurityEvaluator::audit_log(
                            &self.ws,
                            "filesystem.write",
                            &format!("write_to_file {:?}", safe_path),
                            true,
                        );

                        if let Some(parent) = safe_path.parent() {
                            std::fs::create_dir_all(parent).unwrap_or_default();
                        }

                        if std::fs::write(&safe_path, content).is_ok() {
                            self.make_success(id, json!({ "content": [{ "type": "text", "text": "File written successfully" }] }))
                        } else {
                            self.make_error(id, -32002, "Failed to write file")
                        }
                    }
                    Err(e) => {
                        SecurityEvaluator::audit_log(
                            &self.ws,
                            "filesystem.write",
                            &format!("write_to_file denied: {}", e),
                            false,
                        );
                        self.make_error(id, -32003, &e)
                    }
                }
            }

            "search_paths" => {
                let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
                if let Ok(matches) = self.search_mgr.search_paths(query) {
                    let paths: Vec<String> = matches.into_iter().map(|m| m.path).collect();
                    self.make_success(
                        id,
                        json!({ "content": [{ "type": "text", "text": paths.join("\n") }] }),
                    )
                } else {
                    self.make_error(id, -32002, "Path search failed")
                }
            }

            "search_symbols" => {
                let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
                if let Ok(matches) = self.search_mgr.search_symbols(query) {
                    let sym_lines: Vec<String> = matches
                        .into_iter()
                        .map(|m| format!("{} ({}): line {}", m.name, m.kind, m.start_line))
                        .collect();
                    self.make_success(
                        id,
                        json!({ "content": [{ "type": "text", "text": sym_lines.join("\n") }] }),
                    )
                } else {
                    self.make_error(id, -32002, "Symbol search failed")
                }
            }

            "search_code" => {
                let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
                if let Ok(matches) = self.search_mgr.search_code(query) {
                    let results: Vec<String> = matches
                        .into_iter()
                        .map(|m| format!("{}:L{} -> {}", m.path, m.line, m.content))
                        .collect();
                    self.make_success(
                        id,
                        json!({ "content": [{ "type": "text", "text": results.join("\n") }] }),
                    )
                } else {
                    self.make_error(id, -32002, "Code search failed")
                }
            }

            "get_context" => {
                let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
                let budget = args
                    .get("token_budget")
                    .and_then(|b| b.as_u64())
                    .unwrap_or(2000) as usize;

                match ContextEngine::assemble_context(&self.ws, &self.search_mgr, query, budget) {
                    Ok(profile) => {
                        let response_text =
                            serde_json::to_string_pretty(&profile).unwrap_or_default();
                        self.make_success(
                            id,
                            json!({ "content": [{ "type": "text", "text": response_text }] }),
                        )
                    }
                    Err(e) => self.make_error(id, -32002, &e),
                }
            }

            _ => self.make_error(id, -32601, "Method not found"),
        }
    }

    fn make_success(&self, id: Value, result: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        })
    }

    fn make_error(&self, id: Value, code: i32, message: &str) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn setup_temp_workspace() -> PathBuf {
        let path = std::env::temp_dir().join(format!("mcp_runtime_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path.canonicalize().unwrap()
    }

    #[tokio::test]
    async fn test_mcp_initialize() {
        let root = setup_temp_workspace();
        let server = McpServer::new(root.clone()).unwrap();

        let response = server
            .handle_request(json!(1), "initialize", json!({}))
            .await;
        assert_eq!(response.get("id").unwrap().as_i64().unwrap(), 1);
        let result = response.get("result").unwrap();
        assert_eq!(
            result.get("protocolVersion").unwrap().as_str().unwrap(),
            "2024-11-05"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn test_mcp_list_tools() {
        let root = setup_temp_workspace();
        let server = McpServer::new(root.clone()).unwrap();

        let response = server
            .handle_request(json!("req-2"), "tools/list", json!({}))
            .await;
        assert_eq!(response.get("id").unwrap().as_str().unwrap(), "req-2");
        let result = response.get("result").unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();

        // Assert we exposed our key tools
        assert!(tools
            .iter()
            .any(|t| t.get("name").unwrap().as_str().unwrap() == "list_dir"));
        assert!(tools
            .iter()
            .any(|t| t.get("name").unwrap().as_str().unwrap() == "view_file"));
        assert!(tools
            .iter()
            .any(|t| t.get("name").unwrap().as_str().unwrap() == "write_to_file"));
        assert!(tools
            .iter()
            .any(|t| t.get("name").unwrap().as_str().unwrap() == "get_context"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn test_mcp_call_tool_write_and_view() {
        let root = setup_temp_workspace();
        let server = McpServer::new(root.clone()).unwrap();

        // Write file
        let write_resp = server
            .handle_request(
                json!(3),
                "tools/call",
                json!({
                    "name": "write_to_file",
                    "arguments": {
                        "path": "test.txt",
                        "content": "MCP is awesome!"
                    }
                }),
            )
            .await;

        assert!(write_resp.get("error").is_none());
        assert!(write_resp.get("result").is_some());

        // View file
        let view_resp = server
            .handle_request(
                json!(4),
                "tools/call",
                json!({
                    "name": "view_file",
                    "arguments": {
                        "path": "test.txt"
                    }
                }),
            )
            .await;

        assert!(view_resp.get("error").is_none());
        let result = view_resp.get("result").unwrap();
        let content = result.get("content").unwrap().as_array().unwrap();
        assert_eq!(
            content[0].get("text").unwrap().as_str().unwrap(),
            "MCP is awesome!"
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
