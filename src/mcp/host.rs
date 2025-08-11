use crate::mcp::client::{McpClient, McpToolDescription};
use crate::mcp::config::{EnvVar, McpConfig, McpServerConfig};
use anyhow::{Context, Result};
use std::{collections::HashMap, process::Stdio};
use tokio::process::Command;

pub struct McpHost {
    clients: HashMap<String, McpClient>,
    pub tools: HashMap<String, (String /*server*/, McpToolDescription)>,
}

impl McpHost {
    pub async fn from_config(cfg: McpConfig) -> Result<Self> {
        let mut clients = HashMap::new();
        for s in cfg.servers {
            if let Ok(client) = spawn_server(&s).await {
                clients.insert(s.name.clone(), client);
            }
        }

        // Initialize clients and gather tools
        let mut tools = HashMap::new();
        for (name, client) in clients.iter_mut() {
            if let Err(e) = client.initialize().await {
                eprintln!("[MCP] initialize failed for {}: {}", name, e);
                continue;
            }
            match client.list_tools().await {
                Ok(list) => {
                    for t in list {
                        tools.insert(t.name.clone(), (name.clone(), t));
                    }
                }
                Err(e) => eprintln!("[MCP] tools/list failed for {}: {}", name, e),
            }
        }

        Ok(Self { clients, tools })
    }

    pub async fn call(&mut self, tool: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        let (server, _desc) = self.tools.get(tool).context("Unknown tool")?.clone();
        let client = self.clients.get_mut(&server).context("Server not found")?;
        client.call_tool(tool, args).await
    }
}

async fn spawn_server(cfg: &McpServerConfig) -> Result<McpClient> {
    let mut cmd = Command::new(&cfg.command);
    cmd.args(&cfg.args);
    if let Some(cwd) = &cfg.cwd { cmd.current_dir(cwd); }
    for EnvVar { key, value } in &cfg.env { cmd.env(key, value); }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit());

    let mut child = cmd.spawn().with_context(|| format!("Failed to start MCP server {}", cfg.name))?;
    let stdin = child.stdin.take().context("Failed to open stdin")?;
    let stdout = child.stdout.take().context("Failed to open stdout")?;
    Ok(McpClient::new(cfg.name.clone(), child, stdin, stdout))
}
