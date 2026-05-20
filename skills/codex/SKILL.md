---
name: ai-history
description: |
  Search and browse AI coding assistant chat history across providers (Claude Code, Codex CLI, Cursor).
  Find past conversations, inject historical context into the current session, and search
  across all chat records. Use when asked to find past conversations, load context from
  previous sessions, or search chat history.
license: MIT
metadata:
  author: jiantao88
  tags: ai-history, chat-history, context-sharing, claude, codex, cursor
---

# ai-history

Search and browse AI chat history across Claude Code, Codex CLI, and Cursor.

## Prerequisites

The `ai-history` binary must be installed at `~/.cargo/bin/ai-history`.

If not found, guide the user:

```
Run this to install:
  git clone https://github.com/jiantao88/ai-history.git /tmp/ai-history && /tmp/ai-history/setup && rm -rf /tmp/ai-history
```

## Available Commands

### List all projects

```bash
~/.cargo/bin/ai-history list --json
```

### List sessions for a project (fuzzy match)

```bash
~/.cargo/bin/ai-history sessions "<project>" --json
```

### Show a session's messages

```bash
~/.cargo/bin/ai-history show "<session-id>" --compact --json
```

Session IDs support prefix matching (e.g. `a247accc` matches the full UUID).

### Search across all history

```bash
~/.cargo/bin/ai-history search "<query>" --limit 10 --json
```

### Load a session as context (digest — compressed summary)

```bash
~/.cargo/bin/ai-history context "<session-id>"
```

Outputs a structured digest: intent, key decisions, code changes, remaining issues, conclusion.
For full uncompressed conversation, add `--full`.

### Generate a standalone digest

```bash
~/.cargo/bin/ai-history digest "<session-id>" --json
```

### Export a session (full text)

```bash
~/.cargo/bin/ai-history export "<session-id>" --format prompt
```

The `prompt` format outputs clean `User:` / `Assistant:` blocks with no tool calls
or timestamps — ready to inject into the current conversation as context.

## How to Use

1. **Find**: Use `list` → `sessions` → `show` to drill down to a specific conversation
2. **Search**: Use `search` to find conversations by keyword across all providers
3. **Inject context**: Use `context` to load a compressed digest of a past session,
   or `context --full` for the complete conversation

## Output Formatting

- Always use `--json` flag to get structured data, then format it into readable tables
- Abbreviate session IDs to 8 characters in display
- Abbreviate home directory as `~` in displayed paths
- Truncate long text to keep output scannable

## Important Rules

- **Read-only.** Never modify any chat history files.
- **Always use full path** `~/.cargo/bin/ai-history` to invoke the binary.
- **Handle missing binary gracefully.** If the command fails, guide the user to install.
