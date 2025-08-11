use anyhow::{Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    env,
    io::{self, Write},
};
use futures_util::StreamExt;
mod mcp;
use mcp::{config::McpConfig, host::McpHost};

#[derive(Parser)]
#[command(name = "rust-openai-chat")]
#[command(about = "A simple CLI chat tool using Azure OpenAI")]
struct Cli {
    /// Azure OpenAI endpoint URL (can be set via OPENAI_API_ENDPOINT environment variable)
    #[arg(short, long, env = "OPENAI_API_ENDPOINT", hide_env_values = true)]
    endpoint: Option<String>,

    /// API key for authentication (can be set via OPENAI_API_KEY environment variable)
    #[arg(short, long, env = "OPENAI_API_KEY", hide_env_values = true)]
    api_key: Option<String>,

    /// Deployment name/model name (can be set via OPENAI_API_MODEL environment variable)
    #[arg(
        short,
        long,
        env = "OPENAI_API_MODEL",
        default_value = "gpt-35-turbo",
        hide_env_values = true
    )]
    model: String,

    /// Azure OpenAI API version (e.g., 2025-01-01-preview). Can be set via OPENAI_API_VERSION
    #[arg(
        long,
        env = "OPENAI_API_VERSION",
        default_value = "2025-01-01-preview",
        hide_env_values = true
    )]
    api_version: String,

    /// Enable streaming responses (SSE). Set --stream=false to disable.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set, 
        help = "Enable streaming responses (SSE). Set --stream=false to disable.")]
    stream: bool,

    /// Path to MCP configuration file (YAML). If provided, MCP tools can be used.
    #[arg(long, env = "MCP_CONFIG", hide_env_values = true)]
    mcp_config: Option<String>,
}

#[derive(Serialize)]
struct ChatRequest {
    messages: Vec<serde_json::Value>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>, // OpenAI tool definitions
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponseBasic {
    choices: Vec<ChoiceBasic>,
}

#[derive(Deserialize)]
struct ChoiceBasic {
    message: ChatMessage,
}

struct ChatClient {
    client: Client,
    endpoint: String,
    api_key: String,
    model: String,
    api_version: String,
}

impl ChatClient {
    fn new(endpoint: String, api_key: String, model: String, api_version: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            api_key,
            model,
            api_version,
        }
    }

    async fn send_message(&self, messages: &[serde_json::Value]) -> Result<String> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.model, self.api_version
        );

        let request = ChatRequest {
            messages: messages.to_vec(),
            max_tokens: 1000,
            temperature: 0.7,
            tools: None,
            tool_choice: None,
            stream: Some(false),
        };

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Azure OpenAI")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed: {}", error_text);
        }

    let chat_response: ChatResponseBasic = response
            .json()
            .await
            .context("Failed to parse response from Azure OpenAI")?;

        Ok(chat_response
            .choices
            .first()
            .context("No response choices available")?
            .message
            .content
            .clone())
    }

    async fn send_message_streaming(&self, messages: &[serde_json::Value]) -> Result<String> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.model, self.api_version
        );

        let request = ChatRequest {
            messages: messages.to_vec(),
            max_tokens: 1000,
            temperature: 0.7,
            tools: None,
            tool_choice: None,
            stream: Some(true),
        };

    let response = self
            .client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Azure OpenAI (stream)")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed: {}", error_text);
        }

    // Stream Server-Sent Events: lines starting with 'data: '
    let mut body_stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut full_text = String::new();
    let mut done = false;

        // Write prefix once; the caller prints the label.
        while let Some(chunk) = body_stream.next().await {
            let chunk = chunk.context("Failed reading stream chunk")?;
            let s = String::from_utf8_lossy(&chunk);
            buffer.push_str(&s);

            // Process complete lines
        while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim_end().to_string();
                buffer.drain(..pos + 1);

                if line.is_empty() { continue; }

                // Azure sends lines like: "data: {json}" and "data: [DONE]"
                let data_prefix = "data:";
                if let Some(rest) = line.strip_prefix(data_prefix) {
                    let data = rest.trim();
            if data == "[DONE]" { done = true; break; }

                    if let Some(delta) = extract_delta_from_stream_payload(data) {
                        print!("{}", delta);
                        io::stdout().flush().ok();
                        full_text.push_str(&delta);
                    }
                }
            }
            if done { break; }
        }

        // Ensure newline after stream completes
        println!();
        Ok(full_text)
    }

    // Non-streaming call with tools enabled, returns full JSON value
    async fn send_with_tools(&self, messages: &[serde_json::Value], tools: &[serde_json::Value]) -> Result<serde_json::Value> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.model, self.api_version
        );

        let request = ChatRequest {
            messages: messages.to_vec(),
            max_tokens: 1000,
            temperature: 0.7,
            tools: Some(tools.to_vec()),
            tool_choice: Some(serde_json::json!({"type":"auto"})),
            stream: Some(false),
        };

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Azure OpenAI (tools)")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed: {}", error_text);
        }

        let v: serde_json::Value = response.json().await.context("Failed to parse tools response")?;
        Ok(v)
    }
}

/// Extract the incremental content delta from a single SSE JSON payload string.
/// Returns Some(content) if choices[0].delta.content exists and is non-empty.
fn extract_delta_from_stream_payload(data: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(data).ok()?;
    let s = v
        .get("choices")?
        .get(0)?
        .get("delta")?
        .get("content")?
        .as_str()?;
    if s.is_empty() { None } else { Some(s.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_delta_content() {
        let payload = r#"{"choices":[{"delta":{"content":"Hello"}}]}"#;
        assert_eq!(extract_delta_from_stream_payload(payload), Some("Hello".to_string()));
    }

    #[test]
    fn ignores_noncontent() {
        let payload = r#"{"choices":[{"delta":{"role":"assistant"}}]}"#;
        assert_eq!(extract_delta_from_stream_payload(payload), None);
    }

    #[test]
    fn accumulates_sequence() {
        let parts = vec![
            r#"{"choices":[{"delta":{"content":"Hel"}}]}"#,
            r#"{"choices":[{"delta":{"content":"lo"}}]}"#,
            r#"{"choices":[{"delta":{"content":"!"}}]}"#,
        ];
        let mut s = String::new();
        for p in parts {
            if let Some(x) = extract_delta_from_stream_payload(p) { s.push_str(&x); }
        }
        assert_eq!(s, "Hello!");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Read required configuration; error out if neither CLI args nor env vars provide them
    let endpoint = cli.endpoint
        .or_else(|| env::var("OPENAI_API_ENDPOINT").ok())
        .context("Azure OpenAI endpoint is required. Provide it via --endpoint argument or OPENAI_API_ENDPOINT environment variable")?;

    let api_key = cli.api_key
        .or_else(|| env::var("OPENAI_API_KEY").ok())
        .context("API key is required. Provide it via --api-key argument or OPENAI_API_KEY environment variable")?;

    let model = if cli.model == "gpt-35-turbo" {
        env::var("OPENAI_API_MODEL").unwrap_or_else(|_| cli.model)
    } else {
        cli.model
    };

    let chat_client = ChatClient::new(endpoint, api_key, model, cli.api_version.clone());

    // Load MCP config and start servers (non-blocking best-effort)
    let mut mcp_host: Option<McpHost> = None;
    if let Some(cfg_path) = &cli.mcp_config {
        match McpConfig::load_from_path(cfg_path) {
            Ok(cfg) => {
                match McpHost::from_config(cfg).await {
                    Ok(host) => {
                        mcp_host = Some(host);
                        eprintln!("[MCP] Loaded servers and tools.");
                    }
                    Err(e) => eprintln!("[MCP] Failed to start servers: {}", e),
                }
            }
            Err(e) => eprintln!("[MCP] Failed to load config: {}", e),
        }
    }
    let mut conversation: Vec<serde_json::Value> = vec![serde_json::json!({
        "role":"system",
        "content":"You are a helpful assistant."
    })];

    println!("🤖 Azure OpenAI Chat CLI");
    println!("Type 'quit' or 'exit' to end the conversation.");
    println!("Type 'clear' to clear the conversation history.");
    println!("{}", "=".repeat(50));

    loop {
    // Read user input from prompt
        let user_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("You")
            .interact_text()
            .context("Failed to read user input")?;

    // Handle special commands
        match user_input.trim().to_lowercase().as_str() {
            "quit" | "exit" => {
                println!("👋 Goodbye!");
                break;
            }
            "clear" => {
                conversation.clear();
                conversation.push(serde_json::json!({"role":"system","content":"You are a helpful assistant."}));
                println!("🗑️ Conversation cleared!");
                continue;
            }
            _ if user_input.trim().is_empty() => continue,
            _ => {}
        }

    // Append user message to the conversation history
    conversation.push(serde_json::json!({"role":"user","content": user_input}));

    // Show a "thinking" indicator
        print!("🤖 Assistant: ");
        io::stdout().flush().unwrap();
        if !cli.stream {
            print!("thinking...\r");
            io::stdout().flush().unwrap();
        }

    // Send request to Azure OpenAI (MVP: no tool-call loop yet)
        let result = if cli.stream && mcp_host.is_none() {
            chat_client.send_message_streaming(&conversation).await
        } else if mcp_host.is_none() {
            chat_client.send_message(&conversation).await
        } else {
            // With MCP enabled, run non-streaming tool-call loop
            // Build tool definitions from MCP
            let mut host = mcp_host.as_mut().unwrap();
            let tools: Vec<serde_json::Value> = host.tools.values().map(|(_server, desc)| {
                serde_json::json!({
                    "type":"function",
                    "function":{
                        "name": desc.name,
                        "description": desc.description.clone().unwrap_or_default(),
                        "parameters": desc.input_schema
                    }
                })
            }).collect();

            let mut local_conv = conversation.clone();
            let final_text = loop {
                let resp = chat_client.send_with_tools(&local_conv, &tools).await?;
                let choice = &resp["choices"][0]["message"];
                // Append assistant message (may have tool_calls)
                local_conv.push(choice.clone());
                if let Some(tool_calls) = choice.get("tool_calls").and_then(|v| v.as_array()) {
                    for tc in tool_calls {
                        let id = tc["id"].as_str().unwrap_or_default();
                        let func = &tc["function"];
                        let name = func["name"].as_str().unwrap_or("");
                        let args_str = func["arguments"].as_str().unwrap_or("{}");
                        let args_json: serde_json::Value = serde_json::from_str(args_str).unwrap_or(serde_json::json!({"raw": args_str}));
                        let tool_result = host.call(name, args_json).await.unwrap_or(serde_json::json!({"error":"tool call failed"}));
                        local_conv.push(serde_json::json!({
                            "role":"tool",
                            "tool_call_id": id,
                            "content": serde_json::to_string(&tool_result).unwrap_or("null".to_string())
                        }));
                    }
                    // Continue loop to let model consume tool outputs
                    continue;
                } else {
                    // No tool calls; return content
                    let content = choice.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
                    break content;
                }
            };

            // Update real conversation with latest assistant text
            Ok(final_text)
        };

        match result {
            Ok(response) => {
                // For non-streaming mode: clear "thinking..." and print reply
                if !cli.stream {
                    print!("\r🤖 Assistant: {}\n", response);
                }

                // Append assistant reply to conversation history
                conversation.push(serde_json::json!({"role":"assistant","content": response}));
            }
            Err(e) => {
                println!("\r❌ Error: {}", e);
                // On error, remove the last user message from history
                conversation.pop();
            }
        }

        println!();
    }

    Ok(())
}
