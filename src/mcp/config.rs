use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpConfig {
    /// List of MCP servers to start/connect.
    pub servers: Vec<McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpServerConfig {
    /// A human-friendly name.
    pub name: String,
    /// Command to start the MCP server (stdio transport expected).
    pub command: String,
    /// Arguments for the command.
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional environment variables for the server process.
    #[serde(default)]
    pub env: Vec<EnvVar>,
    /// Optional working directory.
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

impl McpConfig {
    pub fn load_from_path(path: &str) -> Result<Self> {
        let s = fs::read_to_string(path)
            .with_context(|| format!("Failed to read MCP config from {}", path))?;
        let cfg: McpConfig = serde_yaml::from_str(&s)
            .with_context(|| format!("Invalid MCP config YAML in {}", path))?;
        Ok(cfg)
    }
}
