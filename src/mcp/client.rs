use anyhow::{anyhow, Context, Result};
use serde_json::json;
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, process::{Child, ChildStdin, ChildStdout}};
use tokio_util::codec::{FramedRead, LinesCodec};

#[derive(Debug)]
pub struct McpClient {
    pub name: String,
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    id_counter: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDescription {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: serde_json::Value,
}

impl McpClient {
    pub fn new(name: String, child: Child, stdin: ChildStdin, stdout: ChildStdout) -> Self {
        Self { name, child, stdin, stdout, id_counter: 0 }
    }

    fn next_id(&mut self) -> u64 { self.id_counter += 1; self.id_counter }

    pub async fn initialize(&mut self) -> Result<()> {
        // Minimal MCP initialize over JSON-RPC
        let id = self.next_id();
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "clientInfo": {"name": "rust-openai-chat", "version": env!("CARGO_PKG_VERSION")}
            }
        });
        self.send(req).await?;
        let _resp = self.read().await?; // TODO: validate
        Ok(())
    }

    pub async fn list_tools(&mut self) -> Result<Vec<McpToolDescription>> {
        let id = self.next_id();
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/list",
            "params": {}
        });
        self.send(req).await?;
        let resp = self.read().await?;
        let tools = resp["result"]["tools"].as_array()
            .ok_or_else(|| anyhow!("Invalid tools/list response"))?
            .iter()
            .map(|t| McpToolDescription {
                name: t["name"].as_str().unwrap_or("").to_string(),
                description: t.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                input_schema: t.get("inputSchema").cloned().unwrap_or(serde_json::json!({"type":"object"})),
            })
            .collect();
        Ok(tools)
    }

    pub async fn call_tool(&mut self, name: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        let id = self.next_id();
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {"name": name, "arguments": args}
        });
        self.send(req).await?;
        let resp = self.read().await?;
        Ok(resp["result"].clone())
    }

    async fn send(&mut self, value: serde_json::Value) -> Result<()> {
        let s = serde_json::to_string(&value)?;
        self.stdin.write_all(s.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn read(&mut self) -> Result<serde_json::Value> {
        let mut reader = BufReader::new(&mut self.stdout);
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 { return Err(anyhow!("MCP server closed stdout")); }
        let v: serde_json::Value = serde_json::from_str(&line).context("Invalid JSON-RPC line")?;
        if v.get("error").is_some() { return Err(anyhow!(format!("MCP error: {}", v["error"]))); }
        Ok(v)
    }
}
