import { Tab } from '../types';

interface HeaderProps {
  activeTab: Tab;
}

export default function Header({ activeTab }: HeaderProps) {
  const handleRefresh = () => {
    // Perform full visual refresh of the application
    window.location.reload();
  };

  return (
    <header className="h-14 border-b border-border-subtle flex items-center justify-between px-6 bg-surface-primary/50 backdrop-blur-md">
      <div className="flex items-center space-x-2">
        <span className="text-xs text-text-muted capitalize">WorkspaceOS</span>
        <span className="text-text-muted">/</span>
        <span className="text-xs text-text-primary font-medium capitalize">{activeTab}</span>
      </div>
      <div className="flex items-center space-x-4">
        <button
          onClick={handleRefresh}
          className="p-1.5 rounded-lg border border-border-subtle hover:bg-surface-secondary text-text-secondary transition duration-150 cursor-pointer flex items-center justify-center"
          title="Refresh Application"
        >
          <span className="material-symbols-rounded text-sm">refresh</span>
        </button>
        <div className="h-4 w-px bg-border-subtle" />
        <div className="flex items-center space-x-2 text-xs">
          <span className="material-symbols-rounded text-success-main text-base">
            verified_user
          </span>
          <span className="text-text-secondary font-medium">Core Secure</span>
        </div>
      </div>
    </header>
  );
}
