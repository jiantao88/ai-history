# ai-history

[English](#english) | [中文](#中文)

---

## English

A terminal tool that searches and exports chat history from AI coding assistants. Read conversations from **Claude Code** and **Codex CLI**, output as Markdown, JSON, or clean prompt text — ready to pipe into another AI tool's context.

### Why

Every AI coding session starts from zero. The assistant doesn't know what you discussed yesterday, what decisions were made, or what was tried and failed. `ai-history` bridges that gap: pull a past conversation and feed it into your current session.

```
┌─────────────┐     ┌─────────────┐     ┌──────────────────┐
│ Claude Code  │────▶│             │────▶│  Markdown / JSON  │
│  ~/.claude/  │     │  ai-history │     │  / Prompt format  │
├─────────────┤     │             │     └────────┬─────────┘
│  Codex CLI   │────▶│             │              │
│  ~/.codex/   │     └─────────────┘              ▼
└─────────────┘                          Paste into any AI tool
```

### Supported Providers

| Provider | Data Location | Format |
|----------|--------------|--------|
| Claude Code | `~/.claude/projects/{encoded-path}/*.jsonl` | JSONL with nested content blocks |
| Codex CLI | `~/.codex/sessions/**/rollout-*.jsonl` | Event-based JSONL |

### Installation

```bash
# From source
git clone https://github.com/jiantao88/ai-history.git
cd ai-history
cargo install --path .
```

Requires Rust 1.70+. The binary installs to `~/.cargo/bin/ai-history`.

### Usage

```bash
# List all projects across all providers
ai-history list

# List sessions in a project (fuzzy match)
ai-history sessions myproject

# Show a session's conversation
ai-history show <session-id>
ai-history show <session-id> --compact   # user/assistant messages only

# Search across all chat history
ai-history search "authentication bug" -n 10

# Export a session
ai-history export <session-id> --format md       # Markdown
ai-history export <session-id> --format json     # JSON
ai-history export <session-id> --format prompt   # Clean User:/Assistant: text
```

Session IDs support prefix matching — `a247accc` matches `a247accc-cc7c-4b1e-ae7f-48b2d57a440c`.

### Global Options

```bash
--json                 # Force JSON output (auto-detected when piped)
--provider claude      # Filter to specific provider(s)
--provider claude,codex
```

### Pipe-Friendly

When stdout is not a TTY, output automatically switches to JSON:

```bash
# Inject past context into clipboard
ai-history export <id> --format prompt | pbcopy

# Filter with jq
ai-history list --json | jq '.[] | select(.provider == "codex")'

# Feed into another AI session
ai-history export <id> --format prompt | claude --prompt -
```

### Export Formats

| Format | Flag | Description |
|--------|------|-------------|
| Markdown | `--format md` | Headings per role, session metadata, tool call details |
| JSON | `--format json` | Structured array of all messages |
| Prompt | `--format prompt` | Clean `User:` / `Assistant:` blocks only — no tool calls, no timestamps. Ready to paste into any AI tool |

### Use as a Claude Code Skill

`ai-history` can also be installed as a Claude Code slash command:

1. Copy the skill to `~/.claude/skills/ai-history/`
2. Add routing to your `~/.claude/CLAUDE.md`

Then in any Claude Code session:

```
/ai-history                          # list all projects
/ai-history sessions myproject       # list sessions
/ai-history search "keyword"         # search history
/ai-history context <session-id>     # load past session as context
```

### Architecture

```
src/
  main.rs              # CLI entry + clap dispatch
  cli.rs               # Subcommand definitions (clap derive)
  model.rs             # Provider-agnostic types: Project, Session, Message
  parse.rs             # JSONL parsing: mmap + simd-json + memchr
  search.rs            # Cross-provider search
  provider/
    mod.rs             # Provider trait + registry
    claude.rs          # Claude Code provider
    codex.rs           # Codex CLI provider
  output/
    mod.rs             # TTY detection
    human.rs           # Colored terminal tables
    json.rs            # JSON output
    markdown.rs        # Markdown export
    prompt.rs          # Clean prompt format
```

### Adding a New Provider

1. Create `src/provider/<name>.rs` implementing the `Provider` trait
2. Add `pub mod <name>;` in `src/provider/mod.rs`
3. Register in `build_registry()`

### License

MIT

---

## 中文

一个终端工具，用于搜索和导出 AI 编程助手的聊天记录。读取 **Claude Code** 和 **Codex CLI** 的对话数据，输出为 Markdown、JSON 或干净的 prompt 文本——可以直接管道传入其他 AI 工具的上下文。

### 为什么需要这个工具

每次 AI 编程会话都从零开始。助手不知道你昨天讨论了什么、做了什么决定、尝试过什么方案。`ai-history` 解决了这个问题：把过去的对话拉出来，注入到当前会话中。

```
┌─────────────┐     ┌─────────────┐     ┌──────────────────┐
│ Claude Code  │────▶│             │────▶│  Markdown / JSON  │
│  ~/.claude/  │     │  ai-history │     │  / Prompt 格式    │
├─────────────┤     │             │     └────────┬─────────┘
│  Codex CLI   │────▶│             │              │
│  ~/.codex/   │     └─────────────┘              ▼
└─────────────┘                         粘贴到任意 AI 工具中
```

### 支持的 Provider

| Provider | 数据路径 | 格式 |
|----------|---------|------|
| Claude Code | `~/.claude/projects/{编码路径}/*.jsonl` | JSONL，嵌套 content blocks |
| Codex CLI | `~/.codex/sessions/**/rollout-*.jsonl` | 基于事件的 JSONL |

### 安装

```bash
# 从源码安装
git clone https://github.com/jiantao88/ai-history.git
cd ai-history
cargo install --path .
```

需要 Rust 1.70+。二进制文件安装到 `~/.cargo/bin/ai-history`。

### 使用方法

```bash
# 列出所有项目
ai-history list

# 列出某项目的会话（模糊匹配）
ai-history sessions myproject

# 查看对话内容
ai-history show <session-id>
ai-history show <session-id> --compact   # 仅显示 user/assistant 消息

# 跨 provider 搜索聊天记录
ai-history search "认证 bug" -n 10

# 导出会话
ai-history export <session-id> --format md       # Markdown 格式
ai-history export <session-id> --format json     # JSON 格式
ai-history export <session-id> --format prompt   # 干净的 User:/Assistant: 文本
```

Session ID 支持前缀匹配——输入 `a247accc` 即可匹配完整的 `a247accc-cc7c-4b1e-ae7f-48b2d57a440c`。

### 全局选项

```bash
--json                 # 强制 JSON 输出（管道时自动切换）
--provider claude      # 过滤特定 provider
--provider claude,codex
```

### 管道友好

当 stdout 不是 TTY 时，输出自动切换为 JSON：

```bash
# 把历史上下文复制到剪贴板
ai-history export <id> --format prompt | pbcopy

# 用 jq 过滤
ai-history list --json | jq '.[] | select(.provider == "codex")'

# 注入到另一个 AI 会话
ai-history export <id> --format prompt | claude --prompt -
```

### 导出格式

| 格式 | 参数 | 说明 |
|------|------|------|
| Markdown | `--format md` | 按角色分标题，包含会话元数据和工具调用详情 |
| JSON | `--format json` | 结构化的消息数组 |
| Prompt | `--format prompt` | 只有 `User:` / `Assistant:` 对话块——没有工具调用、没有时间戳。可以直接粘贴到任何 AI 工具中 |

### 作为 Claude Code Skill 使用

`ai-history` 也可以安装为 Claude Code 斜杠命令：

1. 将 skill 复制到 `~/.claude/skills/ai-history/`
2. 在 `~/.claude/CLAUDE.md` 中添加路由规则

然后在任意 Claude Code 会话中使用：

```
/ai-history                          # 列出所有项目
/ai-history sessions myproject       # 列出会话
/ai-history search "关键词"           # 搜索聊天记录
/ai-history context <session-id>     # 将过去的会话作为上下文加载
```

### 项目结构

```
src/
  main.rs              # 入口 + clap 命令分发
  cli.rs               # 子命令定义（clap derive）
  model.rs             # Provider 无关的通用类型：Project, Session, Message
  parse.rs             # JSONL 解析：mmap + simd-json + memchr
  search.rs            # 跨 provider 搜索
  provider/
    mod.rs             # Provider trait + 注册表
    claude.rs          # Claude Code provider
    codex.rs           # Codex CLI provider
  output/
    mod.rs             # TTY 检测
    human.rs           # 彩色终端表格
    json.rs            # JSON 输出
    markdown.rs        # Markdown 导出
    prompt.rs          # 干净的 prompt 格式
```

### 添加新 Provider

1. 创建 `src/provider/<name>.rs`，实现 `Provider` trait
2. 在 `src/provider/mod.rs` 中添加 `pub mod <name>;`
3. 在 `build_registry()` 中注册

### 许可证

MIT
