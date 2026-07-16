import { Tab } from '../types';
import WorkspaceLogo from './icons/WorkspaceLogo';
import { motion } from 'motion/react';

interface SidebarProps {
  activeTab: Tab;
  setActiveTab: (tab: Tab) => void;
  isMcpRunning: boolean;
}

const sidebarItems = [
  { id: 'dashboard', label: 'Dashboard', iconName: 'space_dashboard' },
  { id: 'workspaces', label: 'Workspaces', iconName: 'folder_open' },
  { id: 'mcp', label: 'MCP Server', iconName: 'dns' },
  { id: 'settings', label: 'Settings', iconName: 'settings' },
  { id: 'logs', label: 'Log Viewer', iconName: 'terminal' },
  { id: 'about', label: 'About', iconName: 'info' },
] as const;

export default function Sidebar({ activeTab, setActiveTab, isMcpRunning }: SidebarProps) {
  return (
    <aside className="w-64 border-r border-border-subtle bg-surface-primary flex flex-col justify-between p-4">
      <div>
        {/* Header Branding with Animated Vector Logo */}
        <div className="flex items-center space-x-3 px-2 py-3 mb-6">
          <WorkspaceLogo className="w-8 h-8" />
          <div>
            <h1 className="font-semibold text-text-primary leading-tight">WorkspaceOS</h1>
            <span className="text-[10px] text-text-muted font-medium tracking-wide uppercase">
              AI Runtime
            </span>
          </div>
        </div>

        {/* Navigation Links */}
        <nav className="space-y-1">
          {sidebarItems.map((item) => {
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => setActiveTab(item.id)}
                className={`w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-150 relative cursor-pointer ${
                  isActive
                    ? 'text-text-primary font-medium bg-surface-secondary'
                    : 'text-text-secondary hover:text-text-primary hover:bg-surface-secondary/50'
                }`}
              >
                {isActive && (
                  <motion.div
                    layoutId="active-sidebar"
                    className="absolute left-0 w-1 h-5 rounded-r bg-accent-primary"
                    transition={{ type: 'spring', stiffness: 380, damping: 30 }}
                  />
                )}
                <span
                  className={`material-symbols-rounded ${isActive ? 'text-accent-primary' : 'text-text-muted'}`}
                >
                  {item.iconName}
                </span>
                <span>{item.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      {/* Sidebar Footer */}
      <div className="border-t border-border-subtle pt-4 px-2 space-y-3">
        <div className="flex items-center justify-between text-xs text-text-muted">
          <span>Status</span>
          <div className="flex items-center space-x-1.5">
            <span
              className={`w-2 h-2 rounded-full ${isMcpRunning ? 'bg-success-main animate-pulse' : 'bg-danger-main'}`}
            />
            <span>{isMcpRunning ? 'Online' : 'Offline'}</span>
          </div>
        </div>
        <div className="text-[10px] text-text-muted flex justify-between">
          <span>v1.0.0 (Beta)</span>
          <span>Local Engine</span>
        </div>
      </div>
    </aside>
  );
}
