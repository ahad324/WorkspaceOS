export default function AboutView() {
  return (
    <div className="space-y-6 max-w-2xl">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">About WorkspaceOS</h2>
        <p className="text-sm text-text-secondary">System version details and licensing.</p>
      </div>

      <div className="bg-surface-primary border border-border-subtle rounded-xl p-6 space-y-6 shadow-sm">
        <div className="space-y-3">
          <h3 className="text-base font-semibold border-b border-border-subtle pb-2">
            Architecture Model
          </h3>
          <p className="text-xs text-text-secondary leading-relaxed">
            WorkspaceOS is a high-performance local runtime built from the ground up to securely
            connect LLMs with local development environments. By indexing files and symbol
            hierarchies using tree-sitter, WorkspaceOS builds a localized relational knowledge graph
            of code bases. It allows any MCP-compatible AI client to navigate repositories with
            contextual relevance without consuming excessive token budgets.
          </p>
        </div>

        <div className="grid grid-cols-2 gap-4 text-xs">
          <div className="p-3 bg-bg-app border border-border-subtle rounded-lg">
            <span className="text-text-muted block">Core Runtime</span>
            <span className="font-semibold text-text-primary mt-1 block">
              Rust (Tokio Async, Axum)
            </span>
          </div>
          <div className="p-3 bg-bg-app border border-border-subtle rounded-lg">
            <span className="text-text-muted block">Database Engine</span>
            <span className="font-semibold text-text-primary mt-1 block">
              SQLite + Tantivy Hybrid Search
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
