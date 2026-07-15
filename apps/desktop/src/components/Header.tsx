import { RefreshCw, ShieldCheck } from 'lucide-react';
import { Tab } from '../types';

interface HeaderProps {
  activeTab: Tab;
}

export default function Header({ activeTab }: HeaderProps) {
  return (
    <header className="h-14 border-b border-border-subtle flex items-center justify-between px-6 bg-surface-primary/50 backdrop-blur-md">
      <div className="flex items-center space-x-2">
        <span className="text-xs text-text-muted capitalize">WorkspaceOS</span>
        <span className="text-text-muted">/</span>
        <span className="text-xs text-text-primary font-medium capitalize">{activeTab}</span>
      </div>
      <div className="flex items-center space-x-4">
        <button className="p-1.5 rounded-lg border border-border-subtle hover:bg-surface-secondary text-text-secondary transition duration-150">
          <RefreshCw className="w-3.5 h-3.5" />
        </button>
        <div className="h-4 w-px bg-border-subtle" />
        <div className="flex items-center space-x-2 text-xs">
          <ShieldCheck className="w-4 h-4 text-success-main" />
          <span className="text-text-secondary font-medium">Core Secure</span>
        </div>
      </div>
    </header>
  );
}
