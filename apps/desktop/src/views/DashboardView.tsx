import { FolderGit2, Cpu, Activity, ShieldCheck, Network, Play, Square } from 'lucide-react';

interface DashboardViewProps {
  isMcpRunning: boolean;
  setIsMcpRunning: (v: boolean) => void;
}

export default function DashboardView({ isMcpRunning, setIsMcpRunning }: DashboardViewProps) {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">Welcome to WorkspaceOS</h2>
        <p className="text-sm text-text-secondary">
          The secure operating layer between your AI models and development directories.
        </p>
      </div>

      {/* Metrics Row */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        {[
          { label: 'Active Workspaces', value: '1', change: 'Isolated', icon: FolderGit2 },
          { label: 'MCP Requests', value: '0', change: 'Idle', icon: Cpu },
          { label: 'Index Health', value: '100%', change: 'Fresh', icon: Activity },
          { label: 'CPU Usage', value: '0.1%', change: 'Minimal', icon: ShieldCheck },
        ].map((stat, i) => {
          const Icon = stat.icon;
          return (
            <div
              key={i}
              className="p-4 bg-surface-primary border border-border-subtle rounded-xl flex flex-col justify-between shadow-sm"
            >
              <div className="flex items-center justify-between">
                <span className="text-xs text-text-muted font-medium">{stat.label}</span>
                <Icon className="w-4 h-4 text-text-muted" />
              </div>
              <div className="mt-2.5">
                <span className="text-2xl font-bold tracking-tight">{stat.value}</span>
                <span className="block text-[10px] text-success-main font-semibold mt-0.5">
                  {stat.change}
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {/* Central control panel & performance graph mockup */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Control Card */}
        <div className="md:col-span-1 p-5 bg-surface-primary border border-border-subtle rounded-xl flex flex-col justify-between shadow-sm">
          <div>
            <h3 className="text-base font-semibold mb-1">Runtime Status</h3>
            <p className="text-xs text-text-muted mb-4">
              Manage the execution of the WorkspaceOS daemon.
            </p>

            <div className="space-y-4">
              <div className="flex justify-between items-center bg-bg-app p-3 rounded-lg border border-border-subtle">
                <div className="flex flex-col">
                  <span className="text-xs font-semibold text-text-primary">MCP Daemon</span>
                  <span className="text-[10px] text-text-muted">localhost:1420</span>
                </div>
                <span
                  className={`px-2 py-0.5 text-[10px] font-bold rounded ${isMcpRunning ? 'bg-success-main/10 text-success-main' : 'bg-text-muted/10 text-text-muted'}`}
                >
                  {isMcpRunning ? 'ACTIVE' : 'INACTIVE'}
                </span>
              </div>

              <div className="flex justify-between items-center bg-bg-app p-3 rounded-lg border border-border-subtle">
                <div className="flex flex-col">
                  <span className="text-xs font-semibold text-text-primary">Tunnel Connection</span>
                  <span className="text-[10px] text-text-muted">Offline</span>
                </div>
                <span className="px-2 py-0.5 text-[10px] font-bold rounded bg-text-muted/10 text-text-muted">
                  DISABLED
                </span>
              </div>
            </div>
          </div>

          <div className="pt-4 flex items-center space-x-2">
            {isMcpRunning ? (
              <button
                onClick={() => setIsMcpRunning(false)}
                className="w-full flex items-center justify-center space-x-2 py-2 px-4 rounded-lg bg-danger-main/10 border border-danger-main/20 hover:bg-danger-main/20 text-danger-main text-xs font-medium transition duration-150"
              >
                <Square className="w-3.5 h-3.5" />
                <span>Stop Runtime</span>
              </button>
            ) : (
              <button
                onClick={() => setIsMcpRunning(true)}
                className="w-full flex items-center justify-center space-x-2 py-2 px-4 rounded-lg bg-accent-primary hover:bg-accent-hover text-white text-xs font-medium transition duration-150"
              >
                <Play className="w-3.5 h-3.5" />
                <span>Start Runtime</span>
              </button>
            )}
          </div>
        </div>

        {/* Running Actions Card */}
        <div className="md:col-span-2 p-5 bg-surface-primary border border-border-subtle rounded-xl flex flex-col justify-between shadow-sm">
          <div>
            <h3 className="text-base font-semibold mb-1">Active Session Monitors</h3>
            <p className="text-xs text-text-muted mb-4">
              No active client sessions found. Connect an AI agent using MCP to get started.
            </p>

            <div className="border border-dashed border-border-subtle rounded-lg flex flex-col items-center justify-center py-10 text-center px-4">
              <Network className="w-8 h-8 text-text-muted mb-2.5" />
              <h4 className="text-xs font-semibold">Waiting for connection...</h4>
              <p className="text-[10px] text-text-muted max-w-xs mt-1">
                Configure your client (like Claude Desktop or cursor-settings) to point to the local
                server endpoint.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
