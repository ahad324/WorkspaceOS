import { useState } from 'react';
import { LogEntry } from '../types';

export default function LogsView() {
  const [logs, setLogs] = useState<LogEntry[]>([
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
          <button
            onClick={() => setLogs([])}
            className="py-1.5 px-3 rounded-lg border border-border-subtle text-xs text-text-secondary hover:bg-surface-secondary transition duration-150"
          >
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
