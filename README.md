# 🌌 WorkspaceOS: Universal AI Workspace Runtime

[![CI Status](https://img.shields.io/badge/CI-passing-success?style=flat-square)](#)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)
[![Version](https://img.shields.io/badge/version-1.0.0-purple?style=flat-square)](#)

WorkspaceOS is a high-performance local runtime that enables modern AI assistants (Grok, ChatGPT, Claude, Gemini, Qwen, DeepSeek) to securely understand, navigate, analyze, and modify local software projects with complete context, absolute security sandbox containment, and low resource overhead.

> [!NOTE]
> WorkspaceOS is not an AI model. It is the intelligence and security layer between AI clients and your local development environment.

---

## 🚀 Key Value Propositions

- **Context Before Tokens**: Instead of feeding thousands of lines of raw code to LLMs, WorkspaceOS calculates relevance-ranked AST code snippets dynamically based on intent, saving API costs and increasing accuracy.
- **Strict Security Sandbox**: Prevents AI tools from traversing directory paths outside the defined workspace boundary. All operations (read, write, search) are audited and restricted.
- **High-Performance FTS**: Powered by Tantivy and Tree-sitter for instantaneous full-text searches, path matching, and symbol resolution without heavy disk scanning.
- **Provider Agnostic**: Exposes capabilities via the standard Model Context Protocol (MCP) to integrate with any client natively.

---

## 🏗️ System Architecture

```mermaid
graph TD
    Client[AI Client / Claude Desktop] -->|JSON-RPC over Stdio/SSE| MCP[MCP Runtime]
    MCP -->|Evaluate Request| Security[Security Engine]
    Security -->|Containment Check & Audit| Workspace[Workspace Engine]
    Workspace -->|Incremental Watcher| Index[Index Engine]
    Workspace -->|Symbol Search & FTS| Search[Search Engine]
    Workspace -->|Relevance Ranking| Context[Context Engine]
    Search -->|SQL & Tantivy Store| DB[(SQLite & Tantivy)]
```

---

## 🛠️ Technology Stack

- **Core Engine**: Rust
- **Desktop Dashboard**: Tauri v2 & React 19
- **Database**: SQLite (persisting symbols & files) + Tantivy (FTS indices)
- **Watcher**: Notify (cross-platform debounced fs-watcher)
- **Parsing**: Tree-sitter AST parser
- **IPC Protocol**: Tauri Event Bridge & command invoking
- **MCP Server**: Stdio / JSON-RPC 2.0 router

---

## 📂 Repository Structure

- `apps/desktop/`: React frontend and Tauri desktop client shell.
- `crates/workspace-engine/`: Handles workspace registrations, configurations, and watcher events.
- `crates/index-engine/`: Extracts syntax symbols (functions, structs) using tree-sitter.
- `crates/search-engine/`: SQLite and Tantivy Full-Text Search.
- `crates/context-engine/`: Detects intent and ranks snippet relevance.
- `crates/security-engine/`: Validates path containment and writes append-only audit logs.
- `crates/mcp-runtime/`: Exposes standard MCP tools to AI clients.
- `crates/tunnel-manager/`: Coordinates ngrok and Cloudflare secure remote tunnels.
- `crates/plugin-system/`: Dynamically preloads companion plugins and loads external manifests.
- `crates/e2e-tests/`: Full integration testing suite.

---

## 🏁 Getting Started

### 📋 Prerequisites
- **Node.js**: v18+ (with `pnpm` package manager)
- **Rust**: stable edition (with `cargo`)

### 🔧 Installation
1. Install node dependencies:
   ```powershell
   pnpm install
   ```
2. Build workspace and verify unit tests:
   ```powershell
   cargo test --workspace
   ```

### 💻 Running Development Client
Boot up the desktop application developer mode:
```powershell
pnpm dev
```

### 🧪 Running Integration Tests
Execute the end-to-end integration tests:
```powershell
cargo test -p e2e-tests
```

For performance benchmarking:
```powershell
cargo bench -p cache_bench
```

---

## 📦 Production Packaging
To build production installers:
```powershell
pnpm --filter desktop tauri build
```
For more information, please read the [Release & Deployment Guide](file:///g:/Ahad/DesktopApps/WorkspaceOS/RELEASE-GUIDE.md).
