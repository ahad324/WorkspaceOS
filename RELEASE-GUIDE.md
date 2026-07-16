# WorkspaceOS Release & Deployment Guide

This document outlines the pipeline for testing, benchmarking, and packaging WorkspaceOS for production release.

---

## 📋 Pre-Release Checklist

Before creating a release candidate, ensure that all quality controls are successfully completed:

1. **Deterministic Formatting**:
   ```powershell
   pnpm format
   ```
2. **Zero Lint Warnings**:
   ```powershell
   pnpm lint
   ```
3. **Rust Compilation & Clippy**:
   ```powershell
   cargo clippy --all-targets -- -D warnings
   ```
4. **All Tests Passing**:
   ```powershell
   cargo test --workspace
   ```

---

## 🧪 E2E Integration Testing

WorkspaceOS features a comprehensive end-to-end integration test suite exercising the entire ecosystem:
- Workspace registration & state machine transitions
- SQLite AST indexer and incremental file watchers
- Tantivy FTS tokenizers & exact string search matching
- Context Engine AI relevance-ranking & scoring budget
- Security Engine path traversal containment & audit trails
- Dynamic Remote Tunnel provider toggles & metric updates
- MCP standard Stdio runtime server loop

To run the integration tests:
```powershell
cargo test -p e2e-tests
```

---

## 📊 Benchmarking & Performance Profiling

To profile the internal caching layer under high concurrency:
```powershell
cargo bench -p cache_bench
```
This runs micro-benchmarks measuring context assembly throughput and cache eviction under extreme simulated request load.

---

## 📦 Production Packaging & Installers

WorkspaceOS desktop applications are compiled using Tauri. Installers are generated for each platform.

### Windows (MSI & EXE)
1. Verify Tauri v2 environment prerequisites.
2. Execute the production bundle builder:
   ```powershell
   pnpm --filter desktop tauri build
   ```
3. Bundled installer locations:
   - MSI: `src-tauri/target/release/bundle/msi/`
   - EXE: `src-tauri/target/release/bundle/nsis/`

### macOS (DMG & APP)
1. Execute the packaging command:
   ```bash
   pnpm --filter desktop tauri build
   ```
2. Bundle location: `src-tauri/target/release/bundle/dmg/`

### Linux (AppImage & DEB)
1. Execute the packaging command:
   ```bash
   pnpm --filter desktop tauri build
   ```
2. Bundle locations:
   - DEB: `src-tauri/target/release/bundle/deb/`
   - AppImage: `src-tauri/target/release/bundle/appimage/`

---

## 🏷️ Release Tagging Workflow

To cut a new production release:

1. Update version tags in `Cargo.toml` and `package.json` configurations.
2. Commit version changes:
   ```bash
   git add .
   git commit -m "chore(release): bump version to 1.0.0"
   ```
3. Generate a signed SemVer git tag:
   ```bash
   git tag -a v1.0.0 -m "WorkspaceOS Production Release 1.0.0"
   ```
4. Push code and tags to the remote repository:
   ```bash
   git push origin main --tags
   ```
5. The GitHub Actions release CI pipeline will trigger automatically, compile installers for all target architectures, and publish them to GitHub Releases.
