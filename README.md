# Rust OpenAI Chat CLI

A simple Rust command-line chat tool for Azure OpenAI GPT services.

## Features

- Chat with Azure OpenAI GPT models
- Conversation history is preserved
- Friendly interactive CLI UI
- Configurable model parameters
- Clear conversation command
- Async for fast response
- Streaming output (SSE): tokens appear as they are generated

## Requirements

- Rust 1.70+ (install via https://rustup.rs/)

## Quick start

```powershell
# Windows PowerShell: set environment variables
$env:OPENAI_API_ENDPOINT = "https://your-resource.openai.azure.com"
$env:OPENAI_API_KEY = "your-api-key"
$env:OPENAI_API_MODEL = "gpt-35-turbo"
$env:OPENAI_API_VERSION = "2025-01-01-preview"

# Run with streaming (default)
cargo run -- --stream

# Or disable streaming (print full reply at once)
cargo run -- --stream=false
```

Or pass parameters explicitly:

```powershell
cargo run -- --endpoint "https://your-resource.openai.azure.com" --api-key "your-api-key" --model "your-deployment-name" --api-version "2025-01-01-preview" --stream
```

## CLI options

- `--endpoint, -e`: Azure OpenAI endpoint URL (or `OPENAI_API_ENDPOINT`)
- `--api-key, -a`: API key (or `OPENAI_API_KEY`)
- `--model, -m`: Deployment/model name (or `OPENAI_API_MODEL`, default: `gpt-35-turbo`)
- `--api-version`: API version (or `OPENAI_API_VERSION`, default: `2025-01-01-preview`)
- `--stream`: Enable streaming output (SSE). Default `true`. Set `--stream=false` to disable.

Notes
- CLI args override environment variables.
- For streaming, the tool parses SSE `data:` lines and stops on `[DONE]`.

## Packaging (Windows)

Use the provided VS Code tasks or run the PowerShell packaging script:

```powershell
# Build release binary and package a zip into dist/
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/package.ps1
```

Result: `dist/rust-openai-chat-<version>-windows-x64.zip` containing the `.exe`, README, and LICENSE if present.

Recipients should set env vars or pass CLI options before running `rust-openai-chat.exe`.

## Release & tagging workflow

You can bump the version, commit, tag, and push with VS Code tasks (PowerShell under the hood):

- Release: patch – increments 0.0.X
- Release: minor – increments 0.X.0
- Release: major – increments X.0.0

This uses `scripts/release.ps1` to:
1) read current version from Cargo.toml
2) bump it
3) commit with message `chore(release): v<version>` and an AI-written summary of changes in the body
4) create tag `v<version>` with the same summary
5) push commit and tag to the default remote

If you use the provided GitHub Actions `release.yml`, pushing a tag `vX.Y.Z` will build and attach artifacts to the GitHub Release.

Advanced: you can pass a pre-generated summary to the script when running manually:
`pwsh scripts/release.ps1 -Bump patch -Summary "- Add streaming SSE support\n- Update README to English-only"`

## Examples

```powershell
# GPT-3.5 Turbo, streaming
cargo run -- -e "https://myresource.openai.azure.com" -a "abc123..." -m "gpt-35-turbo" --stream

# GPT-4 (if deployed), non-streaming
cargo run -- -e "https://myresource.openai.azure.com" -a "abc123..." -m "gpt-4" --stream=false
```

## Project structure

```
rust-openai-chat/
├── src/
│   └── main.rs
├── Cargo.toml
├── README.md
└── scripts/
	└── package.ps1
```

## Dependencies

- tokio: async runtime
- reqwest: HTTP client
- serde/serde_json: JSON types
- clap: CLI args parsing
- anyhow: error handling
- dialoguer: interactive prompts
- futures-util: stream utilities for SSE

## Troubleshooting

1. Authentication error: check endpoint and API key
2. Network error: ensure connectivity and firewall rules
3. Quota limits: verify Azure OpenAI quota
4. Deployment not found: confirm your deployment name in Azure OpenAI Studio

## License

MIT License

## MCP (Model Context Protocol)

Experimental support for MCP servers over stdio is available.

- Provide a YAML file via `--mcp-config path/to/mcp.yaml` or set env `MCP_CONFIG`.
- The CLI will start the servers, initialize them, and list available tools.
- A future update will let the assistant call tools automatically when the model requests it.

Example `mcp.yaml`:

```yaml
servers:
	- name: files
		command: files-mcp-server
		args: ["--root", "."]
		env:
			- key: RUST_LOG
				value: info
		cwd: .
```
