# ai-history

Search and export AI coding assistant chat history from the terminal.

## Overview

A standalone Rust CLI that reads conversation history from multiple AI coding tools and outputs it in various formats. Designed for injecting historical context into another AI tool's conversation.

## Supported Providers

| Provider | Data Location |
|----------|--------------|
| Claude Code | `~/.claude/projects/{encoded-path}/*.jsonl` |
| Codex CLI | `~/.codex/sessions/**/rollout-*.jsonl` |

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# List all projects across providers
ai-history list

# List sessions in a project (fuzzy match)
ai-history sessions rnproject

# Display a session's messages
ai-history show <session-id>
ai-history show <session-id> --compact   # user/assistant only

# Search across all history
ai-history search "keyword" -n 10

# Export a session
ai-history export <session-id> --format md
ai-history export <session-id> --format json
ai-history export <session-id> --format prompt
```

### Global Options

```bash
--json                 # Force JSON output (also auto-detected when piped)
--provider claude      # Filter to specific provider(s)
```

### Pipe-Friendly

When stdout is not a TTY (e.g., piped to another command), output automatically switches to JSON. Use `--format prompt` for human-readable piped output:

```bash
# Inject context into clipboard
ai-history export <id> --format prompt | pbcopy

# Filter with jq
ai-history list --json | jq '.[] | select(.provider == "codex")'
```

## Export Formats

- **md** — Markdown with session metadata, headings per role, tool call details
- **json** — Structured JSON array of messages
- **prompt** — Clean `User:` / `Assistant:` blocks, no tool calls or timestamps. Ready to paste into another AI tool.

## Development

```bash
cargo build          # Debug build
cargo test           # Run tests
cargo build --release  # Release build (with LTO + strip)
```

## License

MIT
