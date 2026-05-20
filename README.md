# ai-history

[中文文档](README_CN.md)

---

Share chat history across AI coding assistants. Search past conversations from **Claude Code**, **Codex CLI**, and **Cursor**, then inject them as context into your current session.

## The Problem

Every AI session starts from zero. Your assistant doesn't remember yesterday's decisions, debugging steps, or architectural choices. You end up re-explaining the same context over and over.

## The Solution

`ai-history` reads chat history from multiple AI tools and makes it available anywhere — as a slash command inside Claude Code or Codex, or as a CLI tool you can pipe into any workflow.

```
┌─────────────┐     ┌─────────────┐     ┌──────────────────┐
│ Claude Code  │────▶│             │────▶│  Markdown / JSON  │
│  ~/.claude/  │     │             │     │  / Prompt format  │
├─────────────┤     │  ai-history │     └────────┬─────────┘
│  Codex CLI   │────▶│             │              │
│  ~/.codex/   │     │             │              ▼
├─────────────┤     │             │     Paste into any AI tool
│   Cursor     │────▶│             │
│  (vscdb)     │     └─────────────┘
└─────────────┘
```

## Architecture

See the interactive architecture diagram: [docs/architecture.html](docs/architecture.html)

## Installation

### One-line install (recommended)

Run this in your terminal, or just tell your AI assistant to run it:

```bash
curl -fsSL https://raw.githubusercontent.com/jiantao88/ai-history/master/setup | bash
```

No Rust, no build tools required — the script downloads a pre-built binary for your platform (macOS ARM64/Intel, Linux x86_64) and installs the `/ai-history` slash command for Claude Code and Codex CLI.

### From source (for developers)

```bash
git clone https://github.com/jiantao88/ai-history.git
cd ai-history
cargo install --path .
./setup                  # install skills only (binary already built)
```

## Use in Claude Code

After installation, use `/ai-history` in any Claude Code session:

```
/ai-history                              # list all projects
/ai-history sessions myproject           # list sessions
/ai-history show <session-id>            # view a conversation
/ai-history search "keyword"             # search across all history
/ai-history context <session-id>         # load digest (compressed summary)
/ai-history context <session-id> --full  # load full conversation
/ai-history context-search "keyword"     # search + auto-load best match
/ai-history digest <session-id>          # generate standalone digest
```

## Use in Codex CLI

After installation, use `/ai-history` in any Codex session:

```
/ai-history                              # list all projects
/ai-history sessions myproject           # list sessions
/ai-history search "keyword"             # search across all history
/ai-history context <session-id>         # load digest from a past session
/ai-history context <session-id> --full  # load full conversation
```

## Use as CLI

You can also use `ai-history` directly in the terminal:

```bash
ai-history list                                    # list projects
ai-history sessions myproject                      # list sessions (fuzzy match)
ai-history show <session-id> --compact             # user/assistant only
ai-history search "auth bug" -n 10                 # search
ai-history context <session-id>                    # digest (compressed summary)
ai-history context <session-id> --full             # full conversation
ai-history digest <session-id>                     # standalone digest
ai-history digest <session-id> --llm               # LLM-enhanced digest
ai-history export <session-id> --format prompt     # export for pasting
ai-history export <session-id> --format md         # Markdown export
ai-history export <session-id> --format json       # JSON export
```

Session IDs support prefix matching — type `a247accc` instead of the full UUID.

### Search Options

```bash
ai-history search "query" -n 20             # limit results
ai-history search "query" -C 2              # show 2 context messages around each match
ai-history search "auth login" --all        # require all terms (AND mode)
ai-history search "query" --sort-time       # sort by time instead of relevance (BM25)
```

### Global Options

```bash
--json                 # Force JSON output (auto-detected when piped)
--provider claude      # Filter to specific provider
--provider claude,codex,cursor
```

### Pipe-Friendly

Output auto-switches to JSON when piped:

```bash
ai-history export <id> --format prompt | pbcopy           # clipboard
ai-history list --json | jq '.[] | select(.provider == "cursor")'
```

## Export Formats

| Format | Flag | Best For |
|--------|------|----------|
| Prompt | `--format prompt` | Pasting into another AI tool — clean `User:` / `Assistant:` blocks, no noise |
| Markdown | `--format md` | Documentation, sharing — includes metadata and tool calls |
| JSON | `--format json` | Programmatic use — full structured message data |

## Supported Providers

| Provider | Data Location |
|----------|--------------|
| Claude Code | `~/.claude/projects/{encoded-path}/*.jsonl` |
| Codex CLI | `~/.codex/sessions/**/rollout-*.jsonl` |
| Cursor | `~/Library/Application Support/Cursor/User/workspaceStorage/*/state.vscdb` (macOS) |

## Adding a New Provider

1. Create `src/provider/<name>.rs` implementing the `Provider` trait
2. Add `pub mod <name>;` in `src/provider/mod.rs`
3. Register in `build_registry()`

## License

MIT
