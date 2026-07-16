import { useState, useEffect } from 'react';
import { listPlugins, PluginMetadata } from '../services/tauriService';
import CopyButton from '../components/CopyButton';

interface McpViewProps {
  isMcpRunning: boolean;
  setIsMcpRunning: (v: boolean) => void;
}

export default function McpView({ isMcpRunning, setIsMcpRunning }: McpViewProps) {
  const [plugins, setPlugins] = useState<PluginMetadata[]>([]);

  useEffect(() => {
    listPlugins().then(setPlugins).catch(console.error);
  }, []);

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
                className={`text-[10px] font-bold px-2.5 py-0.5 rounded border transition-all duration-150 cursor-pointer ${
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
                  name: 'list_dir',
                  desc: 'List the files and directories inside the active workspace.',
                },
                {
                  name: 'view_file',
                  desc: 'View the text content of a file in the workspace with sandbox bounds.',
                },
                {
                  name: 'write_to_file',
                  desc: 'Create or overwrite a file with specific contents.',
                },
                {
                  name: 'search_paths',
                  desc: 'Perform fuzzy search on file paths in the repository.',
                },
                {
                  name: 'search_symbols',
                  desc: 'Search code symbol names (classes, methods, structs) inside files.',
                },
                {
                  name: 'search_code',
                  desc: 'Search occurrences of text inside file bodies.',
                },
                {
                  name: 'get_context',
                  desc: 'Assemble relevance-ranked context profile tailored to query.',
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
            <div className="flex justify-between items-center">
              <h3 className="text-base font-semibold">Client Integration</h3>
              <CopyButton
                value={`{
  "mcpServers": {
    "workspace-os": {
      "command": "workspaceos-core",
      "args": ["mcp", "start"]
    }
  }
}`}
              />
            </div>
            <p className="text-xs text-text-muted">
              Add this configuration to your Claude Desktop config file to connect to WorkspaceOS:
            </p>

            <pre className="p-3 bg-bg-app border border-border-subtle rounded-lg text-[10px] font-mono overflow-x-auto text-text-secondary">
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

          <div className="p-5 bg-surface-primary border border-border-subtle rounded-xl shadow-sm space-y-4">
            <h3 className="text-base font-semibold flex items-center space-x-2">
              <span className="material-symbols-rounded text-text-muted">explore</span>
              <span>Active Workspace Plugins</span>
            </h3>
            <p className="text-xs text-text-muted">
              Loaded extensions enriching the workspace context.
            </p>

            <div className="space-y-3">
              {plugins.length === 0 ? (
                <div className="text-[11px] text-text-muted">No external plugins loaded.</div>
              ) : (
                plugins.map((plugin, idx) => (
                  <div
                    key={idx}
                    className="p-3 bg-bg-app border border-border-subtle rounded-lg space-y-1.5"
                  >
                    <div className="flex justify-between items-center">
                      <span className="text-xs font-semibold text-text-primary">{plugin.name}</span>
                      <span className="text-[9px] px-1.5 py-0.5 rounded bg-surface-secondary text-text-secondary font-mono">
                        v{plugin.version}
                      </span>
                    </div>
                    <p className="text-[10px] text-text-muted">{plugin.description}</p>
                    <div className="flex flex-wrap gap-1 pt-1">
                      {plugin.capabilities.map((cap, cIdx) => (
                        <span
                          key={cIdx}
                          className="text-[8px] font-bold px-1.5 py-0.25 bg-accent-primary/10 text-accent-primary rounded"
                        >
                          {cap}
                        </span>
                      ))}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
