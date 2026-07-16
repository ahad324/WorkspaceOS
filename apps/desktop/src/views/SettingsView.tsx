import { useState, useEffect } from 'react';
import {
  getWorkspaceConfig,
  updateWorkspaceConfig,
  WorkspaceConfig,
} from '../services/tauriService';
import { Shield, Cpu, FolderOpen } from 'lucide-react';

export default function SettingsView() {
  const [config, setConfig] = useState<WorkspaceConfig | null>(null);
  const [saveStatus, setSaveStatus] = useState<string | null>(null);

  useEffect(() => {
    getWorkspaceConfig().then(setConfig).catch(console.error);
  }, []);

  const saveConfig = async (newConfig: WorkspaceConfig) => {
    try {
      await updateWorkspaceConfig(newConfig);
      setConfig(newConfig);
      setSaveStatus('Settings saved successfully!');
      setTimeout(() => setSaveStatus(null), 3000);
    } catch (e) {
      console.error(e);
      setSaveStatus('Failed to save settings.');
    }
  };

  const handleProfileChange = (profile: string) => {
    if (!config) return;
    const newConfig = {
      ...config,
      performance: { ...config.performance, profile },
    };
    saveConfig(newConfig);
  };

  const handleCapabilityToggle = (cap: string) => {
    if (!config) return;
    const currentCaps = config.security.allowed_capabilities;
    const newCaps = currentCaps.includes(cap)
      ? currentCaps.filter((c) => c !== cap)
      : [...currentCaps, cap];

    const newConfig = {
      ...config,
      security: { ...config.security, allowed_capabilities: newCaps },
    };
    saveConfig(newConfig);
  };

  const handleNameChange = (name: string) => {
    if (!config) return;
    const newConfig = {
      ...config,
      general: { ...config.general, name },
    };
    saveConfig(newConfig);
  };

  if (!config) {
    return (
      <div className="flex justify-center items-center h-48 text-text-muted text-xs">
        Loading workspace configuration...
      </div>
    );
  }

  const allCapabilities = [
    { id: 'filesystem.read', label: 'Read Files', desc: 'Allows reading project files' },
    { id: 'filesystem.write', label: 'Write Files', desc: 'Allows modifying/creating files' },
    { id: 'filesystem.delete', label: 'Delete Files', desc: 'Allows deleting workspace files' },
    { id: 'terminal.execute', label: 'Execute Shell', desc: 'Allows running terminal commands' },
    { id: 'git.read', label: 'Read Git', desc: 'Allows inspecting repository history' },
    { id: 'git.write', label: 'Write Git', desc: 'Allows committing and pushing changes' },
  ];

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">System Settings</h2>
          <p className="text-sm text-text-secondary">
            Configure WorkspaceOS global constraints and profiles.
          </p>
        </div>
        {saveStatus && (
          <span className="text-[10px] font-bold px-3 py-1 bg-success-main/10 text-success-main border border-success-main/20 rounded-lg">
            {saveStatus}
          </span>
        )}
      </div>

      <div className="bg-surface-primary border border-border-subtle rounded-xl p-6 max-w-2xl space-y-6 shadow-sm">
        {/* Name input */}
        <div className="space-y-3">
          <h3 className="text-base font-semibold border-b border-border-subtle pb-2 flex items-center space-x-2">
            <FolderOpen className="w-4 h-4 text-text-muted" />
            <span>Workspace Profile</span>
          </h3>
          <div className="flex flex-col space-y-1.5">
            <label className="text-[10px] font-bold text-text-muted">WORKSPACE NAME</label>
            <input
              type="text"
              value={config.general.name}
              onChange={(e) => handleNameChange(e.target.value)}
              className="px-3 py-2 text-xs bg-bg-app border border-border-subtle rounded-lg text-text-primary focus:outline-none focus:ring-1 focus:ring-accent-primary"
            />
          </div>
        </div>

        {/* Profile profile selector */}
        <div className="space-y-4">
          <h3 className="text-base font-semibold border-b border-border-subtle pb-2 flex items-center space-x-2">
            <Cpu className="w-4 h-4 text-text-muted" />
            <span>Performance Profile</span>
          </h3>
          <div className="grid grid-cols-4 gap-3">
            {['LOW', 'MID', 'HIGH', 'ULTRA'].map((profile) => (
              <button
                key={profile}
                onClick={() => handleProfileChange(profile)}
                className={`py-3 px-4 border rounded-xl flex flex-col items-center justify-center transition duration-150 ${
                  config.performance.profile === profile
                    ? 'border-accent-primary bg-accent-primary/5 text-accent-primary font-bold shadow-sm'
                    : 'border-border-subtle hover:bg-surface-secondary text-text-secondary'
                }`}
              >
                <span className="text-sm">{profile}</span>
                <span className="text-[9px] text-text-muted font-normal mt-0.5">
                  {profile === 'LOW' && '4GB RAM'}
                  {profile === 'MID' && '8GB RAM'}
                  {profile === 'HIGH' && '16GB RAM'}
                  {profile === 'ULTRA' && '32GB+ RAM'}
                </span>
              </button>
            ))}
          </div>
        </div>

        {/* Security permissions config */}
        <div className="space-y-4">
          <h3 className="text-base font-semibold border-b border-border-subtle pb-2 flex items-center space-x-2">
            <Shield className="w-4 h-4 text-text-muted" />
            <span>Security Enforcement</span>
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {allCapabilities.map((cap) => (
              <div
                key={cap.id}
                className="flex items-center justify-between p-3 bg-bg-app rounded-lg border border-border-subtle"
              >
                <div className="mr-4">
                  <span className="text-xs font-semibold text-text-primary">{cap.label}</span>
                  <p className="text-[10px] text-text-muted mt-0.5">{cap.desc}</p>
                </div>
                <input
                  type="checkbox"
                  checked={config.security.allowed_capabilities.includes(cap.id)}
                  onChange={() => handleCapabilityToggle(cap.id)}
                  className="rounded border-border-subtle text-accent-primary focus:ring-accent-primary bg-surface-primary w-4 h-4 cursor-pointer"
                />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
