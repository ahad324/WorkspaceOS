import { useState, useEffect } from 'react';
import { getAuditLogs, clearAuditLogs } from '../services/tauriService';

export default function LogsView() {
  const [logs, setLogs] = useState<string[]>([]);
  const [isPaused, setIsPaused] = useState(false);

  useEffect(() => {
    if (isPaused) return;

    // Initial fetch
    getAuditLogs().then(setLogs).catch(console.error);

    // Dynamic audit logs polling loop
    const interval = setInterval(() => {
      getAuditLogs().then(setLogs).catch(console.error);
    }, 2000);

    return () => clearInterval(interval);
  }, [isPaused]);

  return (
    <div className="space-y-6 h-full flex flex-col">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Structured Log Viewer</h2>
          <p className="text-sm text-text-secondary">
            Real-time system diagnostics and security auditing logs.
          </p>
        </div>
        <div className="flex space-x-2">
          <button
            onClick={async () => {
              try {
                await clearAuditLogs();
                setLogs([]);
              } catch (e) {
                console.error(e);
              }
            }}
            className="py-1.5 px-3 rounded-lg border border-border-subtle hover:bg-surface-secondary text-text-secondary text-xs font-medium transition duration-150 cursor-pointer"
          >
            Clear Logs
          </button>
          <button
            onClick={() => setIsPaused(!isPaused)}
            className={`py-1.5 px-3 rounded-lg border border-border-subtle text-xs font-medium transition duration-150 cursor-pointer ${
              isPaused
                ? 'bg-accent-primary border-accent-primary text-white'
                : 'text-text-secondary hover:bg-surface-secondary'
            }`}
          >
            {isPaused ? 'Resume Stream' : 'Pause Stream'}
          </button>
        </div>
      </div>

      <div className="flex-1 bg-surface-primary border border-border-subtle rounded-xl p-4 font-mono text-xs overflow-y-auto flex flex-col shadow-inner min-h-[400px]">
        {logs.length === 0 || (logs.length === 1 && logs[0].includes('empty')) ? (
          <div className="text-text-muted text-center py-20">
            No audit logs captured yet. Try accessing workspace files to trigger events.
          </div>
        ) : (
          logs.map((log, i) => {
            const isDenied = log.includes('DENIED') || log.includes('Violation');
            return (
              <div
                key={i}
                className="flex space-x-4 py-1 hover:bg-surface-secondary/40 px-2 rounded transition duration-100"
              >
                <span
                  className={`font-bold ${isDenied ? 'text-danger-main' : 'text-success-main'}`}
                >
                  {isDenied ? 'AUDIT_FAIL' : 'AUDIT_OK'}
                </span>
                <span className="text-text-secondary">{log}</span>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
