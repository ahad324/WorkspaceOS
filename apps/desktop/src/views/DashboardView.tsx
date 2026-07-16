import { useState, useEffect } from 'react';
import { FolderGit2, Cpu, Activity, ShieldCheck, Network, Play, Square, Globe } from 'lucide-react';
import { Workspace } from '../types';
import {
  getDiagnostics,
  startTunnel,
  stopTunnel,
  PerformanceDiagnostics,
} from '../services/tauriService';
import CopyButton from '../components/CopyButton';
import LoadingSpinner from '../components/LoadingSpinner';

interface DashboardViewProps {
  isMcpRunning: boolean;
  setIsMcpRunning: (v: boolean) => void;
  activeWorkspace: Workspace | null;
}

export default function DashboardView({
  isMcpRunning,
  setIsMcpRunning,
  activeWorkspace,
}: DashboardViewProps) {
  const [diagnostics, setDiagnostics] = useState<PerformanceDiagnostics | null>(null);
  const [tunnelUrl, setTunnelUrl] = useState<string | null>(null);
  const [isTunnelLoading, setIsTunnelLoading] = useState(false);
  const [provider, setProvider] = useState('Cloudflare');
  const [authToken, setAuthToken] = useState('');

  useEffect(() => {
    // Initial fetch
    getDiagnostics().then(setDiagnostics).catch(console.error);

    // Dynamic metrics polling loop
    const interval = setInterval(() => {
      getDiagnostics().then(setDiagnostics).catch(console.error);
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  const handleTunnelToggle = async () => {
    setIsTunnelLoading(true);
    try {
      if (tunnelUrl) {
        await stopTunnel();
        setTunnelUrl(null);
      } else {
        const url = await startTunnel(provider, authToken);
        setTunnelUrl(url);
      }
    } catch (e) {
      console.error(e);
    } finally {
      setIsTunnelLoading(false);
    }
  };

  const formatBytes = (bytes: number) => {
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  };

  const stats = [
    {
      label: 'Active Workspace',
      value: activeWorkspace ? activeWorkspace.name : 'None',
      change: activeWorkspace ? 'Isolated Boundary' : 'Offline',
      icon: FolderGit2,
    },
    {
      label: 'Indexed Documents',
      value: diagnostics ? `${diagnostics.tantivy_document_count} files` : '0',
      change: diagnostics
        ? `Cache: ${(diagnostics.sqlite_cache_hit_ratio * 100).toFixed(0)}% hit`
        : 'Idle',
      icon: Cpu,
    },
    {
      label: 'Memory Footprint',
      value: diagnostics ? formatBytes(diagnostics.memory_rss_bytes) : '0 MB',
      change: 'Daemon RSS',
      icon: Activity,
    },
    {
      label: 'CPU Usage',
      value: diagnostics ? `${(diagnostics.cpu_usage_percent * 100).toFixed(2)}%` : '0.00%',
      change: 'Minimal Overhead',
      icon: ShieldCheck,
    },
  ];

  if (!diagnostics) {
    return <LoadingSpinner text="Gathers real-time performance profiles..." />;
  }

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
        {stats.map((stat, i) => {
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
                <span className="text-xl font-bold tracking-tight truncate block">
                  {stat.value}
                </span>
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
        <div className="md:col-span-1 p-5 bg-surface-primary border border-border-subtle rounded-xl flex flex-col justify-between shadow-sm space-y-4">
          <div>
            <h3 className="text-base font-semibold mb-1">Runtime Status</h3>
            <p className="text-xs text-text-muted mb-4">
              Manage execution of the secure daemon and remote tunnels.
            </p>

            <div className="space-y-3">
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
                  <span className="text-xs font-semibold text-text-primary">Active Directory</span>
                  <span className="text-[10px] text-text-muted truncate max-w-[150px]">
                    {activeWorkspace ? activeWorkspace.root : 'None'}
                  </span>
                </div>
                <div className="flex items-center space-x-2">
                  {activeWorkspace && <CopyButton value={activeWorkspace.root} />}
                  <span
                    className={`px-2 py-0.5 text-[10px] font-bold rounded ${activeWorkspace ? 'bg-success-main/10 text-success-main' : 'bg-text-muted/10 text-text-muted'}`}
                  >
                    {activeWorkspace ? 'BOUNDED' : 'NONE'}
                  </span>
                </div>
              </div>

              {/* Tunnel Configuration Inputs */}
              {!tunnelUrl && (
                <>
                  <div className="flex flex-col space-y-1.5 bg-bg-app p-3 rounded-lg border border-border-subtle">
                    <label className="text-[10px] font-bold text-text-muted">TUNNEL PROVIDER</label>
                    <select
                      value={provider}
                      onChange={(e) => setProvider(e.target.value)}
                      className="bg-surface-primary border border-border-subtle rounded-lg px-2.5 py-1.5 text-xs text-text-primary focus:outline-none"
                    >
                      <option value="Cloudflare">Cloudflare Tunnel</option>
                      <option value="ngrok">ngrok Tunnel</option>
                      <option value="Tailscale">Tailscale Funnel</option>
                    </select>
                  </div>

                  <div className="flex flex-col space-y-1.5 bg-bg-app p-3 rounded-lg border border-border-subtle">
                    <label className="text-[10px] font-bold text-text-muted">
                      AUTH / ACCESS TOKEN
                    </label>
                    <input
                      type="password"
                      placeholder="Paste tunnel token..."
                      value={authToken}
                      onChange={(e) => setAuthToken(e.target.value)}
                      className="bg-surface-primary border border-border-subtle rounded-lg px-2.5 py-1.5 text-xs text-text-primary focus:outline-none placeholder-text-muted"
                    />
                  </div>
                </>
              )}

              <div className="flex justify-between items-center bg-bg-app p-3 rounded-lg border border-border-subtle">
                <div className="flex flex-col">
                  <span className="text-xs font-semibold text-text-primary">Secure Tunnel</span>
                  <span className="text-[10px] text-text-muted truncate max-w-[150px]">
                    {tunnelUrl ? tunnelUrl : 'Disabled'}
                  </span>
                </div>
                <div className="flex items-center space-x-2">
                  {tunnelUrl && <CopyButton value={tunnelUrl} />}
                  <span
                    className={`px-2 py-0.5 text-[10px] font-bold rounded ${tunnelUrl ? 'bg-success-main/10 text-success-main' : 'bg-text-muted/10 text-text-muted'}`}
                  >
                    {tunnelUrl ? 'CONNECTED' : 'DISCONNECTED'}
                  </span>
                </div>
              </div>
            </div>
          </div>

          <div className="pt-2 space-y-2">
            {isMcpRunning ? (
              <button
                onClick={() => setIsMcpRunning(false)}
                className="w-full flex items-center justify-center space-x-2 py-2 px-4 rounded-lg bg-danger-main/10 border border-danger-main/20 hover:bg-danger-main/20 text-danger-main text-xs font-medium transition duration-150 cursor-pointer"
              >
                <Square className="w-3.5 h-3.5" />
                <span>Stop MCP Daemon</span>
              </button>
            ) : (
              <button
                onClick={() => setIsMcpRunning(true)}
                className="w-full flex items-center justify-center space-x-2 py-2 px-4 rounded-lg bg-accent-primary hover:bg-accent-hover text-white text-xs font-medium transition duration-150 cursor-pointer"
              >
                <Play className="w-3.5 h-3.5" />
                <span>Start MCP Daemon</span>
              </button>
            )}

            <button
              onClick={handleTunnelToggle}
              disabled={isTunnelLoading}
              className={`w-full flex items-center justify-center space-x-2 py-2 px-4 rounded-lg text-xs font-medium transition duration-150 border cursor-pointer ${
                tunnelUrl
                  ? 'bg-danger-main/10 border-danger-main/20 hover:bg-danger-main/20 text-danger-main'
                  : 'bg-accent-primary hover:bg-accent-hover text-white'
              } ${isTunnelLoading ? 'opacity-50 cursor-not-allowed' : ''}`}
            >
              <Globe className="w-3.5 h-3.5" />
              <span>
                {isTunnelLoading
                  ? 'Connecting...'
                  : tunnelUrl
                    ? 'Disconnect Remote Tunnel'
                    : 'Connect Remote Tunnel'}
              </span>
            </button>
          </div>
        </div>

        {/* Running Actions Card */}
        <div className="md:col-span-2 p-5 bg-surface-primary border border-border-subtle rounded-xl flex flex-col justify-between shadow-sm">
          <div>
            <h3 className="text-base font-semibold mb-1">Active Session Monitors</h3>
            <p className="text-xs text-text-muted mb-4">
              {activeWorkspace
                ? `Currently monitoring secure workspace: ${activeWorkspace.name}`
                : 'No active workspace loaded. Go to Workspaces tab to register and load one.'}
            </p>

            <div className="border border-dashed border-border-subtle rounded-lg flex flex-col items-center justify-center py-10 text-center px-4">
              <Network className="w-8 h-8 text-text-muted mb-2.5" />
              <h4 className="text-xs font-semibold">
                {tunnelUrl
                  ? `${provider} Tunnel Active`
                  : activeWorkspace
                    ? 'Workspace Bounded & Shielded'
                    : 'Waiting for connection...'}
              </h4>
              <div className="flex items-center justify-center space-x-2 mt-1.5 max-w-sm">
                <p className="text-[10px] text-text-muted leading-relaxed">
                  {tunnelUrl
                    ? `Your MCP server is accessible remotely via ${provider} at: ${tunnelUrl}`
                    : activeWorkspace
                      ? `Secure path validation is active on ${activeWorkspace.root}.`
                      : 'Configure your client (like Claude Desktop or cursor-settings) to point to the local server endpoint.'}
                </p>
                {tunnelUrl && <CopyButton value={tunnelUrl} />}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
