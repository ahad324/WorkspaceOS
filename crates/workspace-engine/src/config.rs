use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub name: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Workspace".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_allowed_capabilities")]
    pub allowed_capabilities: Vec<String>,
}

fn default_allowed_capabilities() -> Vec<String> {
    vec!["filesystem.read".to_string(), "git.read".to_string()]
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_capabilities: default_allowed_capabilities(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default = "default_performance_profile")]
    pub profile: String,
}

fn default_performance_profile() -> String {
    "MID".to_string()
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            profile: default_performance_profile(),
        }
    }
}
