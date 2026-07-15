import React, { useState } from 'react';
import {
  LayoutDashboard,
  FolderGit2,
  Cpu,
  Settings as SettingsIcon,
  Terminal as TerminalIcon,
  Info,
  Activity,
  ShieldCheck,
  Network,
  Blocks,
  RefreshCw,
  LogOut,
  Sliders,
  CheckCircle2,
  AlertTriangle,
  Play,
  Square,
} from 'lucide-react';
import { motion, AnimatePresence } from 'motion/react';

type Tab = 'dashboard' | 'workspaces' | 'mcp' | 'settings' | 'logs' | 'about';

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');
  const [isMcpRunning, setIsMcpRunning] = useState(true);

  // Sidebar items definition
  const sidebarItems = [
    { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { id: 'workspaces', label: 'Workspaces', icon: FolderGit2 },
    { id: 'mcp', label: 'MCP Server', icon: Cpu },
    { id: 'settings', label: 'Settings', icon: Sliders },
    { id: 'logs', label: 'Log Viewer', icon: TerminalIcon },
    { id: 'about', label: 'About', icon: Info },
  ] as const;

  return (
    <div className="flex h-screen w-screen bg-bg-app text-text-primary overflow-hidden font-sans select-none">
      {/* Sidebar */}
      <aside className="w-64 border-r border-border-subtle bg-surface-primary flex flex-col justify-between p-4">
        <div>
          {/* Header Branding */}
          <div className="flex items-center space-x-3 px-2 py-3 mb-6">
            <div className="w-8 h-8 rounded-lg bg-accent-primary flex items-center justify-center font-bold text-white shadow-lg shadow-accent-primary/20">
              W
            </div>
            <div>
              <h1 className="font-semibold text-text-primary leading-tight">WorkspaceOS</h1>
              <span className="text-xs text-text-muted">Universal AI Runtime</span>
            </div>
          </div>

          {/* Navigation Links */}
          <nav className="space-y-1">
            {sidebarItems.map((item) => {
              const Icon = item.icon;
              const isActive = activeTab === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => setActiveTab(item.id)}
                  className={`w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-150 relative ${
                    isActive
                      ? 'text-text-primary font-medium bg-surface-secondary'
                      : 'text-text-secondary hover:text-text-primary hover:bg-surface-secondary/50'
                  }`}
                >
                  {isActive && (
                    <motion.div
                      layoutId="active-sidebar"
                      className="absolute left-0 w-1 h-5 rounded-r bg-accent-primary"
                      transition={{ type: 'spring', stiffness: 380, damping: 30 }}
                    />
                  )}
                  <Icon
                    className={`w-4 h-4 ${isActive ? 'text-accent-primary' : 'text-text-muted'}`}
                  />
                  <span>{item.label}</span>
                </button>
              );
            })}
          </nav>
        </div>

        {/* Sidebar Footer */}
        <div className="border-t border-border-subtle pt-4 px-2 space-y-3">
          <div className="flex items-center justify-between text-xs text-text-muted">
            <span>Status</span>
            <div className="flex items-center space-x-1.5">
              <span
                className={`w-2 h-2 rounded-full ${isMcpRunning ? 'bg-success-main animate-pulse' : 'bg-danger-main'}`}
              />
              <span>{isMcpRunning ? 'Online' : 'Offline'}</span>
            </div>
          </div>
          <div className="text-[10px] text-text-muted flex justify-between">
            <span>v1.0.0 (Beta)</span>
            <span>Tauri v2</span>
          </div>
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="flex-1 flex flex-col min-w-0 bg-bg-app relative">
        {/* Top Header Bar */}
        <header className="h-14 border-b border-border-subtle flex items-center justify-between px-6 bg-surface-primary/50 backdrop-blur-md">
          <div className="flex items-center space-x-2">
            <span className="text-xs text-text-muted capitalize">WorkspaceOS</span>
            <span className="text-text-muted">/</span>
            <span className="text-xs text-text-primary font-medium capitalize">{activeTab}</span>
          </div>
          <div className="flex items-center space-x-4">
            <button className="p-1.5 rounded-lg border border-border-subtle hover:bg-surface-secondary text-text-secondary transition duration-150">
              <RefreshCw className="w-3.5 h-3.5" />
            </button>
            <div className="h-4 w-px bg-border-subtle" />
            <div className="flex items-center space-x-2 text-xs">
              <ShieldCheck className="w-4 h-4 text-success-main" />
              <span className="text-text-secondary font-medium">Core Secure</span>
            </div>
          </div>
        </header>

        {/* Tab Content with animations */}
        <div className="flex-1 overflow-y-auto p-6">
          <AnimatePresence mode="wait">
            <motion.div
              key={activeTab}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.15, ease: 'easeOut' }}
              className="h-full"
            >
              {activeTab === 'dashboard' && (
                <DashboardView isMcpRunning={isMcpRunning} setIsMcpRunning={setIsMcpRunning} />
              )}
              {activeTab === 'workspaces' && <WorkspacesView />}
              {activeTab === 'mcp' && (
                <McpView isMcpRunning={isMcpRunning} setIsMcpRunning={setIsMcpRunning} />
              )}
              {activeTab === 'settings' && <SettingsView />}
              {activeTab === 'logs' && <LogsView />}
              {activeTab === 'about' && <AboutView />}
            </motion.div>
          </AnimatePresence>
        </div>
      </main>
    </div>
  );
}

// -------------------------------------------------------------
// SUB-VIEWS
// -------------------------------------------------------------

function DashboardView({
  isMcpRunning,
  setIsMcpRunning,
}: {
  isMcpRunning: boolean;
  setIsMcpRunning: (v: boolean) => void;
}) {
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

function WorkspacesView() {
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Workspaces</h2>
          <p className="text-sm text-text-secondary">
            Register and configure local code directories.
          </p>
        </div>
        <button className="bg-accent-primary hover:bg-accent-hover text-white text-xs font-semibold py-2 px-4 rounded-lg transition duration-150">
          Register Workspace
        </button>
      </div>

      <div className="bg-surface-primary border border-border-subtle rounded-xl overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="border-b border-border-subtle bg-surface-secondary/35 text-[11px] font-bold text-text-muted uppercase tracking-wider">
              <th className="px-6 py-3.5">Workspace Name</th>
              <th className="px-6 py-3.5">Path</th>
              <th className="px-6 py-3.5">Indexing</th>
              <th className="px-6 py-3.5">Capabilities</th>
              <th className="px-6 py-3.5 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="text-sm divide-y divide-border-subtle">
            <tr>
              <td className="px-6 py-4 font-semibold">WorkspaceOS (Current)</td>
              <td className="px-6 py-4 font-mono text-xs text-text-muted">
                G:\Ahad\DesktopApps\WorkspaceOS
              </td>
              <td className="px-6 py-4">
                <span className="inline-flex items-center space-x-1.5 px-2 py-0.5 rounded text-[10px] font-bold bg-success-main/10 text-success-main">
                  <CheckCircle2 className="w-3 h-3" />
                  <span>Indexed</span>
                </span>
              </td>
              <td className="px-6 py-4 text-xs text-text-muted">All Allowed</td>
              <td className="px-6 py-4 text-right">
                <button className="text-xs text-accent-primary hover:text-accent-hover font-semibold">
                  Configure
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}

function McpView({
  isMcpRunning,
  setIsMcpRunning,
}: {
  isMcpRunning: boolean;
  setIsMcpRunning: (v: boolean) => void;
}) {
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
            <h3 className="text-base font-semibold">Registered Tools</h3>
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

function SettingsView() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">System Settings</h2>
        <p className="text-sm text-text-secondary">
          Configure WorkspaceOS global constraints and profiles.
        </p>
      </div>

      <div className="bg-surface-primary border border-border-subtle rounded-xl p-6 max-w-2xl space-y-6 shadow-sm">
        <div className="space-y-4">
          <h3 className="text-base font-semibold border-b border-border-subtle pb-2">
            Performance Profile
          </h3>
          <div className="grid grid-cols-4 gap-3">
            {['LOW', 'MID', 'HIGH', 'ULTRA'].map((profile) => (
              <button
                key={profile}
                className={`py-3 px-4 border rounded-xl flex flex-col items-center justify-center transition duration-150 ${
                  profile === 'HIGH'
                    ? 'border-accent-primary bg-accent-primary/5 text-accent-primary font-bold shadow-sm'
                    : 'border-border-subtle hover:bg-surface-secondary text-text-secondary'
                }`}
              >
                <span className="text-sm">{profile}</span>
                <span className="text-[9px] text-text-muted font-normal mt-0.5">
                  {profile === 'LOW' && '4GB RAM'}
                  {profile === 'MID' && '8GB RAM'}
                  {profile === 'HIGH' && '16GB RAM'}
                  {profile === 'ULTRA' && '32GB+ RAM'}
                </span>
              </button>
            ))}
          </div>
        </div>

        <div className="space-y-4">
          <h3 className="text-base font-semibold border-b border-border-subtle pb-2">
            Security Enforcement
          </h3>
          <div className="flex items-center justify-between p-3 bg-bg-app rounded-lg border border-border-subtle">
            <div>
              <span className="text-xs font-semibold text-text-primary">
                Confirm Dangerous Tools
              </span>
              <p className="text-[10px] text-text-muted mt-0.5">
                Always prompt user confirmation before writing or modifying files.
              </p>
            </div>
            <input
              type="checkbox"
              defaultChecked
              className="rounded border-border-subtle text-accent-primary focus:ring-accent-primary bg-surface-primary w-4 h-4"
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function LogsView() {
  const [logs, setLogs] = useState([
    { time: '23:55:04', level: 'INFO', msg: 'Initializing WorkspaceOS Core Runtime...' },
    { time: '23:55:04', level: 'INFO', msg: 'Loaded Workspace configuration file from TOML.' },
    { time: '23:55:04', level: 'INFO', msg: 'Successfully initialized SQLite connection pool.' },
    {
      time: '23:55:05',
      level: 'INFO',
      msg: 'Started filesystem watcher on G:\\Ahad\\DesktopApps\\WorkspaceOS',
    },
    {
      time: '23:55:05',
      level: 'INFO',
      msg: 'Repository Index Engine loaded. Verification: SUCCESS.',
    },
    { time: '23:55:05', level: 'INFO', msg: 'Exposing MCP Tool registries on port 1420.' },
  ]);

  return (
    <div className="space-y-6 h-full flex flex-col">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Structured Log Viewer</h2>
          <p className="text-sm text-text-secondary">
            Real-time system diagnostics and auditing logs.
          </p>
        </div>
        <div className="flex space-x-2">
          <button className="py-1.5 px-3 rounded-lg border border-border-subtle text-xs text-text-secondary hover:bg-surface-secondary transition duration-150">
            Pause Stream
          </button>
          <button className="py-1.5 px-3 rounded-lg border border-border-subtle text-xs text-text-secondary hover:bg-surface-secondary transition duration-150">
            Clear Logs
          </button>
        </div>
      </div>

      <div className="flex-1 bg-surface-primary border border-border-subtle rounded-xl p-4 font-mono text-xs overflow-y-auto flex flex-col shadow-inner min-h-[300px]">
        {logs.map((log, i) => (
          <div
            key={i}
            className="flex space-x-4 py-1 hover:bg-surface-secondary/40 px-2 rounded transition duration-100"
          >
            <span className="text-text-muted">{log.time}</span>
            <span className="text-accent-primary font-bold">{log.level}</span>
            <span className="text-text-secondary">{log.msg}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function AboutView() {
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
