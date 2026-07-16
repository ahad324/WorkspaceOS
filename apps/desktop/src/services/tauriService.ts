import { invoke } from '@tauri-apps/api/core';
import { Workspace } from '../types';

const isTauri =
  typeof window !== 'undefined' &&
  ('__TAURI_IPC__' in window || '__TAURI_INTERNALS__' in window || '__TAURI__' in window);

export async function getWorkspaces(): Promise<Workspace[]> {
  if (!isTauri) {
    return [
      {
        id: 'workspace-0',
        name: 'WorkspaceOS (Mock)',
        root: 'G:\\Ahad\\DesktopApps\\WorkspaceOS',
        created_at: Date.now() / 1000,
        last_modified: Date.now() / 1000,
      },
    ];
  }
  return invoke<Workspace[]>('get_workspaces');
}

export async function registerWorkspace(name: string, path: string): Promise<Workspace> {
  if (!isTauri) {
    return {
      id: Math.random().toString(),
      name,
      root: path,
      created_at: Date.now() / 1000,
      last_modified: Date.now() / 1000,
    };
  }
  return invoke<Workspace>('register_workspace', { name, path });
}

export async function activateWorkspace(id: string): Promise<void> {
  if (!isTauri) {
    console.log(`Mock activating workspace: ${id}`);
    return;
  }
  return invoke<void>('activate_workspace', { id });
}

export async function getActiveWorkspace(): Promise<Workspace | null> {
  if (!isTauri) {
    return {
      id: 'workspace-0',
      name: 'WorkspaceOS (Mock)',
      root: 'G:\\Ahad\\DesktopApps\\WorkspaceOS',
      created_at: Date.now() / 1000,
      last_modified: Date.now() / 1000,
    };
  }
  return invoke<Workspace | null>('get_active_workspace');
}

export interface TunnelStatus {
  state: string;
  public_url: string | null;
  provider: string;
  latency_ms: number;
  reconnect_count: number;
}

export interface PluginMetadata {
  name: string;
  version: string;
  description: string;
  capabilities: string[];
}

export interface PerformanceDiagnostics {
  cpu_usage_percent: number;
  memory_rss_bytes: number;
  sqlite_cache_hit_ratio: number;
  sqlite_active_connections: number;
  tantivy_document_count: number;
  active_fs_watchers: number;
  total_indexing_duration_ms: number;
}

export interface ContextSnippet {
  path: string;
  content: string;
  score: number;
  reason: string;
}

export interface ContextProfile {
  intent: string;
  snippets: ContextSnippet[];
  total_tokens: number;
}

export async function startTunnel(provider: string, authToken: string): Promise<string> {
  if (!isTauri) {
    return `https://${provider.toLowerCase()}.workspaceos.dev/mcp-session`;
  }
  return invoke<string>('start_tunnel', { provider, authToken });
}

export async function stopTunnel(): Promise<void> {
  if (!isTauri) {
    return;
  }
  return invoke<void>('stop_tunnel');
}

export async function listPlugins(): Promise<PluginMetadata[]> {
  if (!isTauri) {
    return [
      {
        name: 'git-companion',
        version: '1.0.0',
        description: 'Exposes Git status, commit, and history tools',
        capabilities: ['git.read', 'git.write'],
      },
    ];
  }
  return invoke<PluginMetadata[]>('list_plugins');
}

export async function getDiagnostics(): Promise<PerformanceDiagnostics> {
  if (!isTauri) {
    return {
      cpu_usage_percent: 0.05,
      memory_rss_bytes: 35000000,
      sqlite_cache_hit_ratio: 0.99,
      sqlite_active_connections: 1,
      tantivy_document_count: 50,
      active_fs_watchers: 1,
      total_indexing_duration_ms: 100,
    };
  }
  return invoke<PerformanceDiagnostics>('get_diagnostics');
}

export async function generateContext(
  query: string,
  tokenBudget?: number,
): Promise<ContextProfile> {
  if (!isTauri) {
    return {
      intent: 'ExplainCode',
      snippets: [
        {
          path: 'src/lib.rs',
          content: 'fn main() {}',
          score: 10.0,
          reason: 'Mock result',
        },
      ],
      total_tokens: 15,
    };
  }
  return invoke<ContextProfile>('generate_context', { query, tokenBudget });
}

export interface WorkspaceConfig {
  general: { name: string };
  security: { allowed_capabilities: string[] };
  performance: { profile: string };
}

export async function getWorkspaceConfig(): Promise<WorkspaceConfig> {
  if (!isTauri) {
    return {
      general: { name: 'WorkspaceOS (Mock)' },
      security: { allowed_capabilities: ['filesystem.read', 'git.read'] },
      performance: { profile: 'HIGH' },
    };
  }
  return invoke<WorkspaceConfig>('get_workspace_config');
}

export async function updateWorkspaceConfig(config: WorkspaceConfig): Promise<void> {
  if (!isTauri) {
    console.log('Mock saved config:', config);
    return;
  }
  return invoke<void>('update_workspace_config', { config });
}

export async function pickDirectory(): Promise<string | null> {
  if (!isTauri) {
    return 'G:\\Ahad\\DesktopApps\\WorkspaceOS\\mock-folder';
  }
  return invoke<string | null>('pick_directory');
}

export async function getAuditLogs(): Promise<string[]> {
  if (!isTauri) {
    return [
      '[1784147196] SUCCESS - Action: filesystem.read, Details: view_file src/lib.rs',
      '[1784147198] SUCCESS - Action: filesystem.write, Details: write_to_file src/utils.py',
    ];
  }
  return invoke<string[]>('get_audit_logs');
}
