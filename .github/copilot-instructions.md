<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# Rust OpenAI Chat CLI Project

This is a Rust command-line tool for chatting with Azure OpenAI GPT services.

## Project Status
- [x] Verify that the copilot-instructions.md file in the .github directory is created.
- [x] Clarify Project Requirements - Rust command line tool for chatting with Azure OpenAI GPT services
- [x] Scaffold the Project - Rust project initialized successfully with cargo
- [x] Customize the Project - Project customized with Azure OpenAI chat functionality, dependencies added, and README created
- [x] Install Required Extensions - Installed Rust Analyzer and CodeLLDB extensions for Rust development  
- [x] Compile the Project - Project compiled successfully after installing Visual Studio Build Tools
- [x] Create and Run Task - Created build and run tasks in VS Code tasks.json, build task executed successfully
- [x] Launch the Project - Skipped - CLI tool requires Azure OpenAI credentials from user
- [x] Ensure Documentation is Complete - README.md and copilot-instructions.md files exist and contain current project information

## Project Structure
- `src/main.rs` - Main application with Azure OpenAI integration
- `Cargo.toml` - Project dependencies and configuration
- `README.md` - Comprehensive usage instructions
- `.vscode/tasks.json` - VS Code build and run tasks

## Usage
Run with environment variables: `cargo run` (requires OPENAI_API_ENDPOINT, OPENAI_API_KEY, OPENAI_API_MODEL env vars)
Or with parameters: `cargo run -- --endpoint "your-endpoint" --api-key "your-key" --model "your-model"`
