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
}

#[derive(Serialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
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

    async fn send_message(&self, messages: &[ChatMessage]) -> Result<String> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.model, self.api_version
        );

        let request = ChatRequest {
            messages: messages.to_vec(),
            max_tokens: 1000,
            temperature: 0.7,
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

        let chat_response: ChatResponse = response
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

    async fn send_message_streaming(&self, messages: &[ChatMessage]) -> Result<String> {
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.endpoint, self.model, self.api_version
        );

        let request = ChatRequest {
            messages: messages.to_vec(),
            max_tokens: 1000,
            temperature: 0.7,
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
    let mut conversation: Vec<ChatMessage> = vec![ChatMessage {
        role: "system".to_string(),
        content: "You are a helpful assistant.".to_string(),
    }];

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
                conversation.push(ChatMessage {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                });
                println!("🗑️ Conversation cleared!");
                continue;
            }
            _ if user_input.trim().is_empty() => continue,
            _ => {}
        }

    // Append user message to the conversation history
        conversation.push(ChatMessage {
            role: "user".to_string(),
            content: user_input,
        });

    // Show a "thinking" indicator
        print!("🤖 Assistant: ");
        io::stdout().flush().unwrap();
        if !cli.stream {
            print!("thinking...\r");
            io::stdout().flush().unwrap();
        }

    // Send request to Azure OpenAI
        let result = if cli.stream {
            chat_client.send_message_streaming(&conversation).await
        } else {
            chat_client.send_message(&conversation).await
        };

        match result {
            Ok(response) => {
                // For non-streaming mode: clear "thinking..." and print reply
                if !cli.stream {
                    print!("\r🤖 Assistant: {}\n", response);
                }

                // Append assistant reply to conversation history
                conversation.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response,
                });
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
