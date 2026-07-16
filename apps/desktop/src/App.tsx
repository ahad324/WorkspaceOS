import { useState, useEffect } from 'react';
import Sidebar from './components/Sidebar';
import Header from './components/Header';
import DashboardView from './views/DashboardView';
import WorkspacesView from './views/WorkspacesView';
import McpView from './views/McpView';
import SettingsView from './views/SettingsView';
import LogsView from './views/LogsView';
import AboutView from './views/AboutView';
import { Tab, Workspace } from './types';
import { motion, AnimatePresence } from 'motion/react';
import {
  getWorkspaces,
  getActiveWorkspaces,
  registerWorkspace,
  activateWorkspace,
  deactivateWorkspace,
  unregisterWorkspace,
} from './services/tauriService';

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');
  const [isMcpRunning, setIsMcpRunning] = useState(true);
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [activeWorkspaces, setActiveWorkspaces] = useState<Workspace[]>([]);
  const activeWorkspace = activeWorkspaces.length > 0 ? activeWorkspaces[0] : null;

  const fetchWorkspacesData = async () => {
    try {
      const list = await getWorkspaces();
      setWorkspaces(list);
      const activeList = await getActiveWorkspaces();
      setActiveWorkspaces(activeList);
    } catch (err) {
      console.error('Failed to load workspace information', err);
    }
  };

  useEffect(() => {
    fetchWorkspacesData();
  }, []);

  const handleRegisterWorkspace = async (name: string, path: string) => {
    try {
      await registerWorkspace(name, path);
      await fetchWorkspacesData();
    } catch (err) {
      console.error('Failed to register workspace', err);
      alert(err instanceof Error ? err.message : String(err));
    }
  };

  const handleActivateWorkspace = async (id: string) => {
    try {
      await activateWorkspace(id);
      await fetchWorkspacesData();
    } catch (err) {
      console.error('Failed to activate workspace', err);
      alert(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDeactivateWorkspace = async (id: string) => {
    try {
      await deactivateWorkspace(id);
      await fetchWorkspacesData();
    } catch (err) {
      console.error('Failed to deactivate workspace', err);
      alert(err instanceof Error ? err.message : String(err));
    }
  };

  const handleUnregisterWorkspace = async (id: string) => {
    try {
      await unregisterWorkspace(id);
      await fetchWorkspacesData();
    } catch (err) {
      console.error('Failed to unregister workspace', err);
      alert(err instanceof Error ? err.message : String(err));
    }
  };

  return (
    <div className="flex h-screen w-screen bg-bg-app text-text-primary overflow-hidden font-sans">
      {/* Navigation Sidebar */}
      <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} isMcpRunning={isMcpRunning} />

      {/* Main content viewport */}
      <main className="flex-1 flex flex-col min-w-0 bg-bg-app relative">
        <Header activeTab={activeTab} />

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
                <DashboardView
                  isMcpRunning={isMcpRunning}
                  setIsMcpRunning={setIsMcpRunning}
                  activeWorkspace={activeWorkspace}
                />
              )}
              {activeTab === 'workspaces' && (
                <WorkspacesView
                  workspaces={workspaces}
                  activeWorkspace={activeWorkspace}
                  activeWorkspaces={activeWorkspaces}
                  onRegister={handleRegisterWorkspace}
                  onActivate={handleActivateWorkspace}
                  onDeactivate={handleDeactivateWorkspace}
                  onUnregister={handleUnregisterWorkspace}
                />
              )}
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
