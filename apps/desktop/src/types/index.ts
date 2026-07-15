export type Tab = 'dashboard' | 'workspaces' | 'mcp' | 'settings' | 'logs' | 'about';

export interface Workspace {
  id: string;
  name: string;
  root: string;
  indexed: boolean;
  capabilities: string[];
}

export interface LogEntry {
  time: string;
  level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG';
  msg: string;
}

export interface McpTool {
  name: string;
  desc: string;
  ready: boolean;
}
