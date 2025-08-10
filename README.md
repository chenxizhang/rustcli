# Rust OpenAI Chat CLI

一个使用 Rust 开发的命令行聊天工具，可以与 Azure OpenAI 的 GPT 服务进行交互。

## 功能特性

- ✨ 与 Azure OpenAI GPT 模型实时聊天
- 💬 保持对话上下文历史
- 🎨 彩色交互式用户界面
- 🔧 可配置的模型参数
- 🧹 支持清除对话历史
- ⚡ 异步处理，响应迅速

## 安装和编译

确保你的系统已安装 Rust（版本 1.70+）。如果没有安装，可以从 [rustup.rs](https://rustup.rs/) 安装。

```bash
# 克隆或进入项目目录
cd rust-openai-chat

# 编译项目
cargo build --release

# 或者直接运行
cargo run -- --help
```

## 如何分发给他人（无需安装 Cargo）

你可以直接分发已编译的二进制文件（或压缩包），接收者无需安装 Rust/Cargo。

### 方式A：本地打包（Windows）
1. 在本机构建发布版并打包：
	 - 在 VS Code 任务里运行：Build Rust Project (Release)
	 - 然后运行：Package (Windows)
2. 生成的压缩包位于 `dist/` 目录，例如：`rust-openai-chat-<版本>-windows-x64.zip`
3. 把该 zip 发给同事，对方解压后直接运行 `rust-openai-chat.exe` 即可。

注意：对方仍需在其环境中设置必要的环境变量（OPENAI_API_ENDPOINT、OPENAI_API_KEY、OPENAI_API_MODEL、OPENAI_API_VERSION）或在命令行传参。

### 方式B：GitHub Releases 自动构建
仓库已包含 GitHub Actions 工作流（`.github/workflows/release.yml`）。

发布步骤：
1. 打 tag（例如 `v0.1.0`）并推送到 GitHub
2. Actions 会自动在 Windows / Linux / macOS 上构建发布产物并上传到 Release
3. 到 GitHub Releases 页面下载对应平台的压缩包，发给用户即可

### 接收者如何运行
- Windows：解压后双击 `rust-openai-chat.exe`，或在 PowerShell 中：
	```powershell
	$env:OPENAI_API_ENDPOINT = "https://your-resource.openai.azure.com"
	$env:OPENAI_API_KEY = "your-api-key"
	$env:OPENAI_API_MODEL = "gpt-35-turbo"
	$env:OPENAI_API_VERSION = "2025-01-01-preview"
	.\rust-openai-chat.exe
	```
- macOS / Linux：下载对应的 tar.gz，解压后在终端运行（需赋予可执行权限 `chmod +x`）。

如需静态链接（减少运行时依赖），可根据目标平台启用相应的静态/筋斗云 C 运行时配置，具体取决于构建环境与公司分发策略。

## 使用方法

### 方法1：使用环境变量（推荐）

设置环境变量后，可以直接运行程序而无需每次输入参数：

**Windows PowerShell:**
```powershell
$env:OPENAI_API_ENDPOINT = "https://your-resource.openai.azure.com"
$env:OPENAI_API_KEY = "your-api-key"
$env:OPENAI_API_MODEL = "gpt-35-turbo"
$env:OPENAI_API_VERSION = "2025-01-01-preview"

# 直接运行，无需额外参数
cargo run
```

**Windows Command Prompt:**
```cmd
set OPENAI_API_ENDPOINT=https://your-resource.openai.azure.com
set OPENAI_API_KEY=your-api-key
set OPENAI_API_MODEL=gpt-35-turbo

cargo run
```

**Linux/Mac:**
```bash
export OPENAI_API_ENDPOINT="https://your-resource.openai.azure.com"
export OPENAI_API_KEY="your-api-key"
export OPENAI_API_MODEL="gpt-35-turbo"
export OPENAI_API_VERSION="2025-01-01-preview"

cargo run
```

### 方法2：使用命令行参数

```bash
cargo run -- --endpoint "https://your-resource.openai.azure.com" --api-key "your-api-key" --model "your-deployment-name" --api-version "2025-01-01-preview"
```

### 参数说明

- `--endpoint` (`-e`): Azure OpenAI 资源的端点 URL（可通过 `OPENAI_API_ENDPOINT` 环境变量设置）
- `--api-key` (`-a`): 用于身份验证的 API 密钥（可通过 `OPENAI_API_KEY` 环境变量设置）
- `--model` (`-m`): 部署名称/模型名称（可通过 `OPENAI_API_MODEL` 环境变量设置，默认: gpt-35-turbo）
- `--api-version`: Azure OpenAI API 版本（可通过 `OPENAI_API_VERSION` 环境变量设置，默认: 2025-01-01-preview）

**注意**: 命令行参数的优先级高于环境变量。如果同时设置了命令行参数和环境变量，将使用命令行参数的值。

### 获取 Azure OpenAI 配置信息

1. **端点 URL**: 在 Azure 门户中，进入你的 Azure OpenAI 资源，在"键和端点"部分找到端点
2. **API 密钥**: 同样在"键和端点"部分找到密钥
3. **部署名称**: 在 Azure OpenAI Studio 中，查看你的模型部署名称

### 交互命令

在聊天过程中，你可以使用以下命令：

- `quit` 或 `exit`: 退出聊天
- `clear`: 清除对话历史重新开始

## 示例

### 使用环境变量（推荐方式）
```powershell
# 设置环境变量
$env:OPENAI_API_ENDPOINT = "https://myresource.openai.azure.com"
$env:OPENAI_API_KEY = "abc123..."
$env:OPENAI_API_MODEL = "gpt-35-turbo"

# 直接运行
cargo run
```

### 使用命令行参数
```bash
# 使用 GPT-3.5 Turbo 模型
cargo run -- -e "https://myresource.openai.azure.com" -a "abc123..." -m "gpt-35-turbo"

# 使用 GPT-4 模型（如果已部署）
cargo run -- -e "https://myresource.openai.azure.com" -a "abc123..." -m "gpt-4"
```

### 混合使用
```powershell
# 设置基本环境变量
$env:OPENAI_API_ENDPOINT = "https://myresource.openai.azure.com"
$env:OPENAI_API_KEY = "abc123..."

# 运行时指定不同的模型
cargo run -- --model "gpt-4"
```

## 项目结构

```
rust-openai-chat/
├── src/
│   └── main.rs          # 主要应用程序代码
├── Cargo.toml           # 项目配置和依赖
├── README.md            # 项目文档
└── .gitignore          # Git 忽略文件
```

## 依赖项

- `tokio`: 异步运行时
- `reqwest`: HTTP 客户端
- `serde`: 序列化/反序列化
- `clap`: 命令行参数解析
- `anyhow`: 错误处理
- `dialoguer`: 交互式命令行界面

## 注意事项

- 确保你的 Azure OpenAI 资源已正确配置并且有可用的配额
- API 密钥应当保密，不要在代码中硬编码或提交到版本控制系统
- **推荐使用环境变量设置敏感信息如API密钥**，这样更安全且方便
- 环境变量名称必须完全匹配：`OPENAI_API_ENDPOINT`、`OPENAI_API_KEY`、`OPENAI_API_MODEL`
- 命令行参数的优先级高于环境变量
- 本工具会在内存中保持对话历史，长时间对话可能会消耗较多 tokens

## 错误排查

1. **认证错误**: 检查 API 密钥和端点 URL 是否正确
2. **网络错误**: 确保网络连接正常，防火墙没有阻止请求
3. **配额不足**: 检查 Azure OpenAI 资源的使用配额
4. **模型不存在**: 确认部署名称在 Azure OpenAI Studio 中存在

## 开发

如果你想修改或扩展这个工具：

```bash
# 在开发模式下运行
cargo run -- [参数]

# 运行测试（如果有）
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

## 许可证

MIT License - 详见 LICENSE 文件
