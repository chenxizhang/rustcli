use anyhow::{Context, Result};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    env,
    io::{self, Write},
};

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
}

#[derive(Serialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 获取必需的配置，如果命令行参数或环境变量都没有提供则报错
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

    let chat_client = ChatClient::new(endpoint, api_key, model, cli.api_version);
    let mut conversation: Vec<ChatMessage> = vec![ChatMessage {
        role: "system".to_string(),
        content: "You are a helpful assistant.".to_string(),
    }];

    println!("🤖 Azure OpenAI Chat CLI");
    println!("Type 'quit' or 'exit' to end the conversation.");
    println!("Type 'clear' to clear the conversation history.");
    println!("{}", "=".repeat(50));

    loop {
        // 获取用户输入
        let user_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("You")
            .interact_text()
            .context("Failed to read user input")?;

        // 处理特殊命令
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

        // 添加用户消息到对话历史
        conversation.push(ChatMessage {
            role: "user".to_string(),
            content: user_input,
        });

        // 显示"正在思考"指示器
        print!("🤖 Assistant: ");
        io::stdout().flush().unwrap();
        print!("thinking...\r");
        io::stdout().flush().unwrap();

        // 发送请求到Azure OpenAI
        match chat_client.send_message(&conversation).await {
            Ok(response) => {
                // 清除"thinking..."提示并显示回复
                print!("\r🤖 Assistant: {}\n", response);

                // 将助手回复添加到对话历史
                conversation.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response,
                });
            }
            Err(e) => {
                println!("\r❌ Error: {}", e);
                // 如果出错，从对话历史中移除用户的最后一条消息
                conversation.pop();
            }
        }

        println!();
    }

    Ok(())
}
