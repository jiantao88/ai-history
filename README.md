# ai-history

[English](#english) | [中文](#中文)

---

## English

Share chat history across AI coding assistants. Search past conversations from **Claude Code** and **Codex CLI**, then inject them as context into your current session.

### The Problem

Every AI session starts from zero. Your assistant doesn't remember yesterday's decisions, debugging steps, or architectural choices. You end up re-explaining the same context over and over.

### The Solution

`ai-history` reads chat history from multiple AI tools and makes it available anywhere — as a slash command inside Claude Code or Codex, or as a CLI tool you can pipe into any workflow.

```
┌─────────────┐     ┌─────────────┐     ┌──────────────────┐
│ Claude Code  │────▶│             │────▶│  Markdown / JSON  │
│  ~/.claude/  │     │  ai-history │     │  / Prompt format  │
├─────────────┤     │             │     └────────┬─────────┘
│  Codex CLI   │────▶│             │              │
│  ~/.codex/   │     └─────────────┘              ▼
└─────────────┘                          Paste into any AI tool
```

### Installation

#### One-line install (recommended)

Just tell your AI assistant to run this:

```
Install ai-history: git clone https://github.com/jiantao88/ai-history.git /tmp/ai-history && /tmp/ai-history/setup && rm -rf /tmp/ai-history
```

The setup script automatically:
- Installs Rust (if not present)
- Builds and installs the `ai-history` binary
- Installs the `/ai-history` slash command for **Claude Code**
- Installs the `/ai-history` skill for **Codex CLI** (if installed)

#### Manual install

```bash
git clone https://github.com/jiantao88/ai-history.git
cd ai-history
./setup                  # full install (binary + skills)
./setup --claude-only    # Claude Code only
./setup --codex-only     # Codex CLI only
```

### Use in Claude Code

After installation, use `/ai-history` in any Claude Code session:

```
/ai-history                              # list all projects
/ai-history sessions myproject           # list sessions
/ai-history show <session-id>            # view a conversation
/ai-history search "keyword"             # search across all history
/ai-history context <session-id>         # load a past session as context
/ai-history context-search "keyword"     # search + auto-load best match
```

### Use in Codex CLI

After installation, use `/ai-history` in any Codex session:

```
/ai-history                              # list all projects
/ai-history sessions myproject           # list sessions
/ai-history search "keyword"             # search across all history
/ai-history context <session-id>         # load context from a past session
```

### Use as CLI

You can also use `ai-history` directly in the terminal:

```bash
ai-history list                                    # list projects
ai-history sessions myproject                      # list sessions (fuzzy match)
ai-history show <session-id> --compact             # user/assistant only
ai-history search "auth bug" -n 10                 # search
ai-history export <session-id> --format prompt     # export for pasting
ai-history export <session-id> --format md         # Markdown export
ai-history export <session-id> --format json       # JSON export
```

Session IDs support prefix matching — type `a247accc` instead of the full UUID.

#### Global Options

```bash
--json                 # Force JSON output (auto-detected when piped)
--provider claude      # Filter to specific provider
--provider claude,codex
```

#### Pipe-Friendly

Output auto-switches to JSON when piped:

```bash
ai-history export <id> --format prompt | pbcopy           # clipboard
ai-history list --json | jq '.[] | select(.provider == "codex")'
```

### Export Formats

| Format | Flag | Best For |
|--------|------|----------|
| Prompt | `--format prompt` | Pasting into another AI tool — clean `User:` / `Assistant:` blocks, no noise |
| Markdown | `--format md` | Documentation, sharing — includes metadata and tool calls |
| JSON | `--format json` | Programmatic use — full structured message data |

### Supported Providers

| Provider | Data Location |
|----------|--------------|
| Claude Code | `~/.claude/projects/{encoded-path}/*.jsonl` |
| Codex CLI | `~/.codex/sessions/**/rollout-*.jsonl` |

### Adding a New Provider

1. Create `src/provider/<name>.rs` implementing the `Provider` trait
2. Add `pub mod <name>;` in `src/provider/mod.rs`
3. Register in `build_registry()`

### License

MIT

---

## 中文

跨 AI 编程助手共享聊天记录。搜索 **Claude Code** 和 **Codex CLI** 的历史对话，然后注入到当前会话的上下文中。

### 问题

每次 AI 会话都从零开始。助手不记得昨天做了什么决定、调试了什么 bug、选择了什么架构方案。你不得不一遍又一遍地重复解释相同的上下文。

### 解决方案

`ai-history` 读取多个 AI 工具的聊天记录，让你可以在任何地方使用它们——作为 Claude Code 或 Codex 中的斜杠命令，或者作为可以管道到任何工作流的 CLI 工具。

```
┌─────────────┐     ┌─────────────┐     ┌──────────────────┐
│ Claude Code  │────▶│             │────▶│  Markdown / JSON  │
│  ~/.claude/  │     │  ai-history │     │  / Prompt 格式    │
├─────────────┤     │             │     └────────┬─────────┘
│  Codex CLI   │────▶│             │              │
│  ~/.codex/   │     └─────────────┘              ▼
└─────────────┘                         粘贴到任意 AI 工具中
```

### 安装

#### 一键安装（推荐）

直接告诉你的 AI 助手执行这条命令：

```
安装 ai-history：git clone https://github.com/jiantao88/ai-history.git /tmp/ai-history && /tmp/ai-history/setup && rm -rf /tmp/ai-history
```

安装脚本会自动完成：
- 安装 Rust（如果没有）
- 编译并安装 `ai-history` 二进制文件
- 为 **Claude Code** 安装 `/ai-history` 斜杠命令
- 为 **Codex CLI** 安装 `/ai-history` skill（如果已安装 Codex）

#### 手动安装

```bash
git clone https://github.com/jiantao88/ai-history.git
cd ai-history
./setup                  # 完整安装（二进制 + skills）
./setup --claude-only    # 仅 Claude Code
./setup --codex-only     # 仅 Codex CLI
```

### 在 Claude Code 中使用

安装后，在任意 Claude Code 会话中使用 `/ai-history`：

```
/ai-history                              # 列出所有项目
/ai-history sessions myproject           # 列出会话
/ai-history show <session-id>            # 查看对话
/ai-history search "关键词"               # 搜索所有聊天记录
/ai-history context <session-id>         # 加载历史会话作为上下文
/ai-history context-search "关键词"       # 搜索并自动加载最相关的会话
```

### 在 Codex CLI 中使用

安装后，在任意 Codex 会话中使用 `/ai-history`：

```
/ai-history                              # 列出所有项目
/ai-history sessions myproject           # 列出会话
/ai-history search "关键词"               # 搜索所有聊天记录
/ai-history context <session-id>         # 加载历史会话上下文
```

### 作为 CLI 使用

也可以直接在终端使用 `ai-history`：

```bash
ai-history list                                    # 列出项目
ai-history sessions myproject                      # 列出会话（模糊匹配）
ai-history show <session-id> --compact             # 仅 user/assistant
ai-history search "认证 bug" -n 10                 # 搜索
ai-history export <session-id> --format prompt     # 导出用于粘贴
ai-history export <session-id> --format md         # Markdown 导出
ai-history export <session-id> --format json       # JSON 导出
```

Session ID 支持前缀匹配——输入 `a247accc` 无需完整 UUID。

#### 全局选项

```bash
--json                 # 强制 JSON 输出（管道时自动切换）
--provider claude      # 过滤特定 provider
--provider claude,codex
```

#### 管道友好

管道输出时自动切换为 JSON：

```bash
ai-history export <id> --format prompt | pbcopy           # 复制到剪贴板
ai-history list --json | jq '.[] | select(.provider == "codex")'
```

### 导出格式

| 格式 | 参数 | 适用场景 |
|------|------|----------|
| Prompt | `--format prompt` | 粘贴到其他 AI 工具——干净的 `User:` / `Assistant:` 对话块，无噪音 |
| Markdown | `--format md` | 文档、分享——包含元数据和工具调用 |
| JSON | `--format json` | 程序化使用——完整的结构化消息数据 |

### 支持的 Provider

| Provider | 数据路径 |
|----------|---------|
| Claude Code | `~/.claude/projects/{编码路径}/*.jsonl` |
| Codex CLI | `~/.codex/sessions/**/rollout-*.jsonl` |

### 添加新 Provider

1. 创建 `src/provider/<name>.rs`，实现 `Provider` trait
2. 在 `src/provider/mod.rs` 中添加 `pub mod <name>;`
3. 在 `build_registry()` 中注册

### 许可证

MIT
