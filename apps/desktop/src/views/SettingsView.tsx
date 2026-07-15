export default function SettingsView() {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold tracking-tight">System Settings</h2>
        <p className="text-sm text-text-secondary">Configure WorkspaceOS global constraints and profiles.</p>
      </div>

      <div className="bg-surface-primary border border-border-subtle rounded-xl p-6 max-w-2xl space-y-6 shadow-sm">
        <div className="space-y-4">
          <h3 className="text-base font-semibold border-b border-border-subtle pb-2">Performance Profile</h3>
          <div className="grid grid-cols-4 gap-3">
            {['LOW', 'MID', 'HIGH', 'ULTRA'].map((profile) => (
              <button
                key={profile}
                className={`py-3 px-4 border rounded-xl flex flex-col items-center justify-center transition duration-150 ${
                  profile === 'HIGH'
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

        <div className="space-y-4">
          <h3 className="text-base font-semibold border-b border-border-subtle pb-2">Security Enforcement</h3>
          <div className="flex items-center justify-between p-3 bg-bg-app rounded-lg border border-border-subtle">
            <div>
              <span className="text-xs font-semibold text-text-primary">Confirm Dangerous Tools</span>
              <p className="text-[10px] text-text-muted mt-0.5">Always prompt user confirmation before writing or modifying files.</p>
            </div>
            <input type="checkbox" defaultChecked className="rounded border-border-subtle text-accent-primary focus:ring-accent-primary bg-surface-primary w-4 h-4" />
          </div>
        </div>
      </div>
    </div>
  );
}
