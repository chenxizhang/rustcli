<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

# Rust OpenAI Chat CLI Project

This is a Rust command-line tool for chatting with Azure OpenAI GPT services.

## Style and Language Guidelines
- All code comments, documentation (including README and any new docs), and commit messages must be written in English.
- When responding in chat or generating explanations, prefer concise English.
- Keep terminology consistent with Azure OpenAI and Rust ecosystems.

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

## Azure OpenAI specifics
- Use the Chat Completions API under `openai/deployments/{deployment}/chat/completions?api-version=...`.
- Support and prefer streaming (SSE) responses by default (request body `{"stream": true}` and `Accept: text/event-stream`).
- For non-streaming mode, omit or set `stream: false` and parse the standard JSON response.
- Maintain a conversation history starting with a `system` message: "You are a helpful assistant." unless instructed otherwise.

## Wrap-up workflow (on user says "收工", "done", or "go")
When the user says any of: "收工", "done", or "go":
1. Save all open files and ensure changes are written to disk.
2. Read `Cargo.toml`, increment the patch version (e.g., 0.1.3 -> 0.1.4), and remember the new version string `vX.Y.Z` for this session.
3. Stage changes and create a commit with subject `chore(release): vX.Y.Z` and include an AI-written concise summary of the changes in the commit body (1–6 short bullets). Base the summary on the session context and git diff, but do NOT paste raw diffs.
4. Create an annotated Git tag `vX.Y.Z`.
	- Include the same AI-written summary in the tag body.
5. Push the commit and the tag to the default Git remote.
6. If a GitHub Actions release workflow exists, note that the tag will trigger the build and release.

Prefer using the integrated terminal commands to run Git operations instead of reinventing scripts. Keep messages and tags in English.
