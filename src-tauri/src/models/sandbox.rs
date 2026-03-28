use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxMode {
    #[default]
    Off,
    #[serde(rename = "non-main")]
    NonMain,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxScope {
    #[default]
    Session,
    Agent,
    Shared,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxWorkspaceAccess {
    #[default]
    None,
    #[serde(rename = "ro")]
    ReadOnly,
    #[serde(rename = "rw")]
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    #[serde(default)]
    pub mode: SandboxMode,
    #[serde(default)]
    pub scope: SandboxScope,
    #[serde(default)]
    pub workspace_access: SandboxWorkspaceAccess,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            mode: SandboxMode::default(),
            scope: SandboxScope::default(),
            workspace_access: SandboxWorkspaceAccess::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxStatus {
    pub enabled: bool,
    pub mode: SandboxMode,
    pub docker_available: bool,
    pub docker_version: Option<String>,
    pub running_containers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSecurityValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}