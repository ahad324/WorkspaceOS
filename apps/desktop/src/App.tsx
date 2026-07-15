import { useState } from 'react';
import Sidebar from './components/Sidebar';
import Header from './components/Header';
import DashboardView from './views/DashboardView';
import WorkspacesView from './views/WorkspacesView';
import McpView from './views/McpView';
import SettingsView from './views/SettingsView';
import LogsView from './views/LogsView';
import AboutView from './views/AboutView';
import { Tab } from './types';
import { motion, AnimatePresence } from 'motion/react';

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('dashboard');
  const [isMcpRunning, setIsMcpRunning] = useState(true);

  return (
    <div className="flex h-screen w-screen bg-bg-app text-text-primary overflow-hidden font-sans select-none">
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
                <DashboardView isMcpRunning={isMcpRunning} setIsMcpRunning={setIsMcpRunning} />
              )}
              {activeTab === 'workspaces' && <WorkspacesView />}
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
