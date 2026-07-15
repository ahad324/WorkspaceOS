import { CheckCircle2 } from 'lucide-react';

export default function WorkspacesView() {
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
