import { 
  LayoutDashboard, 
  FolderGit2, 
  Cpu, 
  Sliders, 
  Terminal as TerminalIcon, 
  Info 
} from 'lucide-react';
import { motion } from 'motion/react';
import { Tab } from '../types';

interface SidebarProps {
  activeTab: Tab;
  setActiveTab: (tab: Tab) => void;
  isMcpRunning: boolean;
}

const sidebarItems = [
  { id: 'dashboard', label: 'Dashboard', icon: LayoutDashboard },
  { id: 'workspaces', label: 'Workspaces', icon: FolderGit2 },
  { id: 'mcp', label: 'MCP Server', icon: Cpu },
  { id: 'settings', label: 'Settings', icon: Sliders },
  { id: 'logs', label: 'Log Viewer', icon: TerminalIcon },
  { id: 'about', label: 'About', icon: Info },
] as const;

export default function Sidebar({ activeTab, setActiveTab, isMcpRunning }: SidebarProps) {
  return (
    <aside className="w-64 border-r border-border-subtle bg-surface-primary flex flex-col justify-between p-4">
      <div>
        {/* Header Branding */}
        <div className="flex items-center space-x-3 px-2 py-3 mb-6">
          <div className="w-8 h-8 rounded-lg bg-accent-primary flex items-center justify-center font-bold text-white shadow-lg shadow-accent-primary/20">
            W
          </div>
          <div>
            <h1 className="font-semibold text-text-primary leading-tight">WorkspaceOS</h1>
            <span className="text-xs text-text-muted">Universal AI Runtime</span>
          </div>
        </div>

        {/* Navigation Links */}
        <nav className="space-y-1">
          {sidebarItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => setActiveTab(item.id)}
                className={`w-full flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-150 relative ${
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
                <Icon className={`w-4 h-4 ${isActive ? 'text-accent-primary' : 'text-text-muted'}`} />
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
            <span className={`w-2 h-2 rounded-full ${isMcpRunning ? 'bg-success-main animate-pulse' : 'bg-danger-main'}`} />
            <span>{isMcpRunning ? 'Online' : 'Offline'}</span>
          </div>
        </div>
        <div className="text-[10px] text-text-muted flex justify-between">
          <span>v1.0.0 (Beta)</span>
          <span>Tauri v2</span>
        </div>
      </div>
    </aside>
  );
}
