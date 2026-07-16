import { useState, useEffect } from 'react';
import { Workspace } from '../types';
import { motion, AnimatePresence } from 'motion/react';
import { pickDirectory } from '../services/tauriService';
import CopyButton from '../components/CopyButton';

interface WorkspacesViewProps {
  workspaces: Workspace[];
  activeWorkspace: Workspace | null;
  activeWorkspaces: Workspace[];
  onRegister: (name: string, path: string) => Promise<void>;
  onActivate: (id: string) => Promise<void>;
  onDeactivate: (id: string) => Promise<void>;
  onUnregister: (id: string) => Promise<void>;
}

function cleanPath(p: string): string {
  if (p.startsWith('\\\\?\\')) return p.slice(4);
  if (p.startsWith('//?/')) return p.slice(4);
  return p;
}

export default function WorkspacesView({
  workspaces,
  activeWorkspaces,
  onRegister,
  onActivate,
  onDeactivate,
  onUnregister,
}: WorkspacesViewProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [name, setName] = useState('');
  const [path, setPath] = useState('');
  const [nameError, setNameError] = useState<string | null>(null);
  const [pathError, setPathError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setError(null);
    setNameError(null);
    setPathError(null);
  }, [name, path, isModalOpen]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setNameError(null);
    setPathError(null);

    const trimmedName = name.trim();
    const trimmedPath = path.trim();
    let hasError = false;

    if (!trimmedName) {
      setNameError('Workspace name is required.');
      hasError = true;
    } else if (trimmedName.length < 2) {
      setNameError('Workspace name must be at least 2 characters.');
      hasError = true;
    }

    if (!trimmedPath) {
      setPathError('Workspace directory path is required.');
      hasError = true;
    }

    if (hasError) return;

    // Verify no duplicate paths registered
    const isDuplicatePath = workspaces.some(
      (ws) => cleanPath(ws.root).toLowerCase() === cleanPath(trimmedPath).toLowerCase(),
    );
    if (isDuplicatePath) {
      setPathError('A workspace with this folder path is already registered.');
      return;
    }

    // Verify no duplicate names registered
    const isDuplicateName = workspaces.some(
      (ws) => ws.name.toLowerCase() === trimmedName.toLowerCase(),
    );
    if (isDuplicateName) {
      setNameError('A workspace with this name is already registered.');
      return;
    }

    onRegister(trimmedName, trimmedPath)
      .then(() => {
        setName('');
        setPath('');
        setIsModalOpen(false);
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : String(err));
      });
  };

  const handleBrowse = async () => {
    const selected = await pickDirectory();
    if (selected) {
      const cleaned = cleanPath(selected);
      setPath(cleaned);
      if (!name) {
        const parts = cleaned.split(/[/\\]/);
        const lastPart = parts[parts.length - 1];
        if (lastPart) {
          setName(lastPart);
        }
      }
    }
  };

  return (
    <div className="space-y-6 relative h-full">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Workspaces</h2>
          <p className="text-sm text-text-secondary">
            Register and configure local code directories.
          </p>
        </div>
        <button
          onClick={() => setIsModalOpen(true)}
          className="bg-accent-primary hover:bg-accent-hover text-white text-xs font-semibold py-2 px-4 rounded-lg flex items-center space-x-1.5 transition duration-150 cursor-pointer"
        >
          <span className="material-symbols-rounded text-sm">add</span>
          <span>Register Workspace</span>
        </button>
      </div>

      <div className="bg-surface-primary border border-border-subtle rounded-xl overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse table-fixed">
          <thead>
            <tr className="border-b border-border-subtle bg-surface-secondary/35 text-[11px] font-bold text-text-muted uppercase tracking-wider">
              <th className="px-6 py-3.5 w-1/4">Workspace Name</th>
              <th className="px-6 py-3.5 w-2/5">Path</th>
              <th className="px-6 py-3.5 w-1/6">Status</th>
              <th className="px-6 py-3.5 w-1/6 text-right">Actions</th>
            </tr>
          </thead>
          <tbody className="text-sm divide-y divide-border-subtle">
            {workspaces.length === 0 ? (
              <tr>
                <td colSpan={4} className="px-6 py-10 text-center text-text-muted text-xs">
                  No registered workspaces. Click "Register Workspace" to add one.
                </td>
              </tr>
            ) : (
              workspaces.map((ws) => {
                const isActive = activeWorkspaces.some((w) => w.id === ws.id);
                const cleanedRoot = cleanPath(ws.root);
                return (
                  <tr key={ws.id} className={isActive ? 'bg-surface-secondary/20' : ''}>
                    <td className="px-6 py-4 font-semibold text-text-primary">
                      <div className="flex items-center space-x-2 truncate">
                        <span className="truncate">{ws.name}</span>
                        {isActive && (
                          <span className="w-1.5 h-1.5 rounded-full bg-success-main animate-pulse" />
                        )}
                      </div>
                    </td>
                    <td className="px-6 py-4 font-mono text-xs text-text-secondary">
                      <div className="flex items-center space-x-2 truncate">
                        <span className="truncate max-w-[280px]" title={cleanedRoot}>
                          {cleanedRoot}
                        </span>
                        <CopyButton value={cleanedRoot} />
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      {isActive ? (
                        <span className="inline-flex items-center space-x-1 px-2 py-0.5 rounded text-[10px] font-bold bg-success-main/10 text-success-main">
                          <span className="material-symbols-rounded text-xs">check_circle</span>
                          <span>Active</span>
                        </span>
                      ) : (
                        <span className="inline-flex items-center space-x-1 px-2 py-0.5 rounded text-[10px] font-bold bg-surface-secondary text-text-muted">
                          <span>Ready</span>
                        </span>
                      )}
                    </td>
                    <td className="px-6 py-4 text-right">
                      <div className="flex items-center justify-end space-x-3.5">
                        {isActive ? (
                          <button
                            onClick={() => onDeactivate(ws.id)}
                            className="text-xs text-text-muted hover:text-text-primary font-semibold transition duration-150 cursor-pointer"
                          >
                            Deactivate
                          </button>
                        ) : (
                          <button
                            onClick={() => onActivate(ws.id)}
                            className="text-xs text-accent-primary hover:text-accent-hover font-semibold transition duration-150 cursor-pointer"
                          >
                            Activate
                          </button>
                        )}
                        <button
                          onClick={() => {
                            if (confirm(`Are you sure you want to unregister "${ws.name}"?`)) {
                              onUnregister(ws.id);
                            }
                          }}
                          className="text-text-muted hover:text-danger-main transition duration-150 cursor-pointer"
                          title="Unregister Workspace"
                        >
                          <span className="material-symbols-rounded text-sm">delete</span>
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>

      {/* Register Modal */}
      <AnimatePresence>
        {isModalOpen && (
          <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <motion.div
              initial={{ scale: 0.95, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.95, opacity: 0 }}
              transition={{ duration: 0.15, ease: 'easeOut' }}
              className="bg-surface-primary border border-border-subtle rounded-2xl max-w-md w-full shadow-2xl p-6 relative overflow-hidden"
            >
              {/* Header */}
              <div className="flex justify-between items-center border-b border-border-subtle pb-3.5 mb-5">
                <div className="flex items-center space-x-2">
                  <span className="material-symbols-rounded text-accent-primary">folder</span>
                  <h3 className="font-semibold text-text-primary">Register Workspace</h3>
                </div>
                <button
                  onClick={() => setIsModalOpen(false)}
                  className="p-1 rounded-lg hover:bg-surface-secondary text-text-muted hover:text-text-primary transition cursor-pointer"
                >
                  <span className="material-symbols-rounded text-sm">close</span>
                </button>
              </div>

              {/* Form */}
              <form onSubmit={handleSubmit} className="space-y-4">
                <div className="flex flex-col space-y-1">
                  <md-outlined-text-field
                    label="Workspace Name"
                    value={name}
                    onInput={(e) => setName((e.target as HTMLInputElement).value)}
                    error={!!nameError || undefined}
                    errorText={nameError || undefined}
                    style={{ width: '100%' }}
                  ></md-outlined-text-field>
                </div>

                <div className="flex flex-col space-y-1">
                  <div className="flex items-end space-x-2">
                    <div className="flex-1">
                      <md-outlined-text-field
                        label="Absolute Path"
                        value={path}
                        onInput={(e) => setPath((e.target as HTMLInputElement).value)}
                        error={!!pathError || undefined}
                        errorText={pathError || undefined}
                        style={{ width: '100%' }}
                      ></md-outlined-text-field>
                    </div>
                    <button
                      type="button"
                      onClick={handleBrowse}
                      className="px-3 py-2 bg-surface-secondary border border-border-subtle rounded-lg hover:bg-border-subtle text-text-primary text-xs font-medium transition cursor-pointer h-[56px]"
                    >
                      Browse...
                    </button>
                  </div>
                </div>

                {error && (
                  <div className="p-2.5 bg-danger-main/10 border border-danger-main/20 text-danger-main rounded-lg text-xs font-medium">
                    {error}
                  </div>
                )}

                <div className="pt-2 flex space-x-3">
                  <button
                    type="button"
                    onClick={() => setIsModalOpen(false)}
                    className="w-1/2 py-2 border border-border-subtle rounded-lg text-xs font-semibold text-text-secondary hover:bg-surface-secondary transition cursor-pointer"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    className="w-1/2 py-2 bg-accent-primary hover:bg-accent-hover text-white rounded-lg text-xs font-semibold transition cursor-pointer"
                  >
                    Register
                  </button>
                </div>
              </form>
            </motion.div>
          </div>
        )}
      </AnimatePresence>
    </div>
  );
}
