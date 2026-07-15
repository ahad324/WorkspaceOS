import { useState } from 'react';
import { CheckCircle2, Plus, X, FolderOpen } from 'lucide-react';
import { Workspace } from '../types';
import { motion, AnimatePresence } from 'motion/react';

interface WorkspacesViewProps {
  workspaces: Workspace[];
  activeWorkspace: Workspace | null;
  onRegister: (name: string, path: string) => Promise<void>;
  onActivate: (id: string) => Promise<void>;
}

export default function WorkspacesView({ 
  workspaces, 
  activeWorkspace, 
  onRegister, 
  onActivate 
}: WorkspacesViewProps) {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [name, setName] = useState('');
  const [path, setPath] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name || !path) return;
    await onRegister(name, path);
    setName('');
    setPath('');
    setIsModalOpen(false);
  };

  return (
    <div className="space-y-6 relative h-full">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Workspaces</h2>
          <p className="text-sm text-text-secondary">Register and configure local code directories.</p>
        </div>
        <button 
          onClick={() => setIsModalOpen(true)}
          className="bg-accent-primary hover:bg-accent-hover text-white text-xs font-semibold py-2 px-4 rounded-lg flex items-center space-x-1.5 transition duration-150"
        >
          <Plus className="w-3.5 h-3.5" />
          <span>Register Workspace</span>
        </button>
      </div>

      <div className="bg-surface-primary border border-border-subtle rounded-xl overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="border-b border-border-subtle bg-surface-secondary/35 text-[11px] font-bold text-text-muted uppercase tracking-wider">
              <th className="px-6 py-3.5">Workspace Name</th>
              <th className="px-6 py-3.5">Path</th>
              <th className="px-6 py-3.5">Status</th>
              <th className="px-6 py-3.5 text-right">Actions</th>
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
                const isActive = activeWorkspace?.id === ws.id;
                return (
                  <tr key={ws.id} className={isActive ? 'bg-surface-secondary/20' : ''}>
                    <td className="px-6 py-4 font-semibold text-text-primary flex items-center space-x-2">
                      <span>{ws.name}</span>
                      {isActive && <span className="w-1.5 h-1.5 rounded-full bg-success-main animate-pulse" />}
                    </td>
                    <td className="px-6 py-4 font-mono text-xs text-text-secondary truncate max-w-[300px]">
                      {ws.root}
                    </td>
                    <td className="px-6 py-4">
                      {isActive ? (
                        <span className="inline-flex items-center space-x-1 px-2 py-0.5 rounded text-[10px] font-bold bg-success-main/10 text-success-main">
                          <CheckCircle2 className="w-3 h-3" />
                          <span>Active</span>
                        </span>
                      ) : (
                        <span className="inline-flex items-center space-x-1 px-2 py-0.5 rounded text-[10px] font-bold bg-surface-secondary text-text-muted">
                          <span>Ready</span>
                        </span>
                      )}
                    </td>
                    <td className="px-6 py-4 text-right">
                      {!isActive && (
                        <button 
                          onClick={() => onActivate(ws.id)}
                          className="text-xs text-accent-primary hover:text-accent-hover font-semibold transition duration-150"
                        >
                          Activate
                        </button>
                      )}
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
                  <FolderOpen className="w-4 h-4 text-accent-primary" />
                  <h3 className="font-semibold text-text-primary">Register Workspace</h3>
                </div>
                <button 
                  onClick={() => setIsModalOpen(false)}
                  className="p-1 rounded-lg hover:bg-surface-secondary text-text-muted hover:text-text-primary transition"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>

              {/* Form */}
              <form onSubmit={handleSubmit} className="space-y-4">
                <div>
                  <label htmlFor="ws-name" className="block text-xs font-semibold text-text-secondary mb-1.5">
                    Workspace Name
                  </label>
                  <input
                    id="ws-name"
                    type="text"
                    required
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="e.g. Inventory System"
                    className="w-full px-3 py-2 text-sm bg-bg-app border border-border-subtle rounded-lg focus:outline-none focus:ring-1 focus:ring-accent-primary focus:border-accent-primary text-text-primary"
                  />
                </div>

                <div>
                  <label htmlFor="ws-path" className="block text-xs font-semibold text-text-secondary mb-1.5">
                    Absolute Path
                  </label>
                  <input
                    id="ws-path"
                    type="text"
                    required
                    value={path}
                    onChange={(e) => setPath(e.target.value)}
                    placeholder="e.g. C:\Projects\InventorySystem"
                    className="w-full px-3 py-2 text-sm bg-bg-app border border-border-subtle rounded-lg focus:outline-none focus:ring-1 focus:ring-accent-primary focus:border-accent-primary text-text-primary font-mono text-xs"
                  />
                </div>

                <div className="pt-2 flex space-x-3">
                  <button
                    type="button"
                    onClick={() => setIsModalOpen(false)}
                    className="w-1/2 py-2 border border-border-subtle rounded-lg text-xs font-semibold text-text-secondary hover:bg-surface-secondary transition"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    className="w-1/2 py-2 bg-accent-primary hover:bg-accent-hover text-white rounded-lg text-xs font-semibold transition"
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
