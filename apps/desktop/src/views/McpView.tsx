interface McpViewProps {
  isMcpRunning: boolean;
  setIsMcpRunning: (v: boolean) => void;
}

export default function McpView({ isMcpRunning, setIsMcpRunning }: McpViewProps) {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">MCP Server Configurations</h2>
        <p className="text-sm text-text-secondary">
          Register and expose core capabilities to Model Context Protocol clients.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="md:col-span-2 space-y-6">
          <div className="p-5 bg-surface-primary border border-border-subtle rounded-xl shadow-sm space-y-4">
            <div className="flex justify-between items-center">
              <h3 className="text-base font-semibold">Registered Tools</h3>
              <button
                onClick={() => setIsMcpRunning(!isMcpRunning)}
                className={`text-[10px] font-bold px-2.5 py-0.5 rounded border transition-all duration-150 ${
                  isMcpRunning
                    ? 'bg-success-main/10 border-success-main/20 text-success-main'
                    : 'bg-danger-main/10 border-danger-main/20 text-danger-main'
                }`}
              >
                {isMcpRunning ? 'RUNNING' : 'STOPPED'}
              </button>
            </div>
            <p className="text-xs text-text-muted">
              The following tools are compiled and ready to be exposed.
            </p>

            <div className="space-y-2">
              {[
                {
                  name: 'workspace_read_file',
                  desc: 'Read code files from workspace root path with strict isolation.',
                },
                { name: 'workspace_write_file', desc: 'Write code files to workspace root path.' },
                {
                  name: 'workspace_search',
                  desc: 'Query full-text indexed contents across repositories.',
                },
                {
                  name: 'workspace_get_symbols',
                  desc: 'Retrieve AST parsed definitions from Tree-sitter.',
                },
              ].map((tool, i) => (
                <div
                  key={i}
                  className="flex justify-between items-start p-3 bg-bg-app rounded-lg border border-border-subtle"
                >
                  <div>
                    <span className="text-xs font-mono font-bold text-accent-primary">
                      {tool.name}
                    </span>
                    <p className="text-[11px] text-text-muted mt-1">{tool.desc}</p>
                  </div>
                  <span className="text-[10px] font-bold text-success-main px-1.5 py-0.5 rounded bg-success-main/10">
                    READY
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div className="md:col-span-1 space-y-6">
          <div className="p-5 bg-surface-primary border border-border-subtle rounded-xl shadow-sm space-y-4">
            <h3 className="text-base font-semibold">Client Integration</h3>
            <p className="text-xs text-text-muted">
              Add this configuration to your Claude Desktop config file to connect to WorkspaceOS:
            </p>

            <pre className="p-3 bg-bg-app border border-border-subtle rounded-lg text-[10px] font-mono overflow-x-auto text-text-secondary select-all">
              {`{
  "mcpServers": {
    "workspace-os": {
      "command": "workspaceos-core",
      "args": ["mcp", "start"]
    }
  }
}`}
            </pre>
          </div>
        </div>
      </div>
    </div>
  );
}
