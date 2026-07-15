import { invoke } from '@tauri-apps/api/core';
import { Workspace } from '../types';

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

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
