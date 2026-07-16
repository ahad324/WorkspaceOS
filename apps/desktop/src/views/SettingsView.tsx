import { useState, useEffect } from 'react';
import {
  getWorkspaceConfig,
  updateWorkspaceConfig,
  WorkspaceConfig,
} from '../services/tauriService';
import LoadingSpinner from '../components/LoadingSpinner';

export default function SettingsView() {
  const [config, setConfig] = useState<WorkspaceConfig | null>(null);
  const [saveStatus, setSaveStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getWorkspaceConfig()
      .then((cfg) => {
        setConfig(cfg);
        setLoading(false);
      })
      .catch((err) => {
        setError(String(err));
        setLoading(false);
      });
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

  if (loading) {
    return <LoadingSpinner text="Retrieving active workspace configuration..." />;
  }

  if (error || !config) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[300px] text-center px-4 space-y-4">
        <div className="p-6 bg-surface-primary border border-border-subtle rounded-2xl max-w-sm shadow-md space-y-3">
          <span className="material-symbols-rounded text-accent-primary text-4xl mx-auto block animate-pulse">
            shield
          </span>
          <h4 className="text-sm font-semibold text-text-primary">No Active Workspace Bounded</h4>
          <p className="text-xs text-text-muted leading-relaxed">
            WorkspaceOS settings are bound to active projects. Please register and activate a
            workspace to configure performance, rules, and capability constraints.
          </p>
        </div>
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
            <span className="material-symbols-rounded text-text-muted">folder</span>
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
            <span className="material-symbols-rounded text-text-muted">memory</span>
            <span>Performance Profile</span>
          </h3>
          <div className="grid grid-cols-4 gap-3">
            {['LOW', 'MID', 'HIGH', 'ULTRA'].map((profile) => (
              <button
                key={profile}
                onClick={() => handleProfileChange(profile)}
                className={`py-3 px-4 border rounded-xl flex flex-col items-center justify-center transition duration-150 cursor-pointer ${
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
            <span className="material-symbols-rounded text-text-muted">shield</span>
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
                <md-checkbox
                  checked={config.security.allowed_capabilities.includes(cap.id)}
                  onClick={() => handleCapabilityToggle(cap.id)}
                  style={{ cursor: 'pointer' }}
                ></md-checkbox>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
