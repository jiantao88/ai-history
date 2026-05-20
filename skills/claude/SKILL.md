---
name: ai-history
version: 0.2.0
description: |
  Search and browse AI coding assistant chat history across providers (Claude Code, Codex CLI, Cursor).
  Find past conversations, inject historical context into the current session, and search
  across all chat records. Powered by the ai-history CLI tool.
allowed-tools:
  - Bash
  - Read
triggers:
  - search chat history
  - find past conversation
  - inject context from history
  - what did we discuss
  - previous session
---

## Overview

This skill provides access to AI coding assistant chat history across multiple providers
(Claude Code, Codex CLI, Cursor). It uses the `ai-history` CLI tool installed at `~/.cargo/bin/ai-history`.

## Prerequisites

The `ai-history` binary must be installed. If not found, guide the user:

```
Run this to install:
  git clone https://github.com/jiantao88/ai-history.git /tmp/ai-history && /tmp/ai-history/setup && rm -rf /tmp/ai-history
```

## Commands

### `/ai-history` or `/ai-history list`

List all projects that have chat history.

```bash
~/.cargo/bin/ai-history list --json 2>/dev/null
```

Parse the JSON output and present as a readable table.

### `/ai-history sessions <project>`

List sessions for a project. The `<project>` argument supports fuzzy matching (substring).

```bash
~/.cargo/bin/ai-history sessions "<project>" --json 2>/dev/null
```

### `/ai-history show <session-id>`

Show messages from a specific session. Session ID can be abbreviated (first 8+ chars).

```bash
~/.cargo/bin/ai-history show "<session-id>" --compact --json 2>/dev/null
```

### `/ai-history search <query>`

Search across all chat history for a keyword or phrase.

```bash
~/.cargo/bin/ai-history search "<query>" --limit 10 --json 2>/dev/null
```

### `/ai-history context <session-id>`

Load a past session as structured context (digest format — compressed summary of intent,
decisions, code changes, and conclusions). This is the recommended way to inject history.

```bash
~/.cargo/bin/ai-history context "<session-id>" 2>/dev/null
```

After fetching:
1. Read and understand the digest
2. Present a brief summary:
   ```
   CONTEXT LOADED (Digest)
   ════════════════════════════════════════
   Session:  <summary>
   Provider: <provider>
   Project:  <project>
   Date:     <date>
   ════════════════════════════════════════
   ```
3. Tell the user you now have the context and can continue the work.

For full uncompressed conversation (when digest isn't enough):

```bash
~/.cargo/bin/ai-history context "<session-id>" --full 2>/dev/null
```

### `/ai-history context-search <query>`

Shortcut: search then automatically load the most relevant session as context.

1. Run search: `~/.cargo/bin/ai-history search "<query>" --limit 5 --json 2>/dev/null`
2. If one clear match, load it directly
3. If multiple, ask the user which session to load

### `/ai-history digest <session-id>`

Generate a standalone digest (compressed summary) of a session.

```bash
~/.cargo/bin/ai-history digest "<session-id>" --json 2>/dev/null
```

## Output Formatting

- Always use `--json` flag when calling `ai-history` to get structured data
- Parse the JSON and format it into readable tables for the user
- Keep output concise — truncate long text
- Abbreviate home directory as `~` in displayed paths

## Important Rules

- **Read-only.** This skill never modifies any chat history files.
- **Always use full path** `~/.cargo/bin/ai-history` to invoke the binary.
- **Handle missing binary gracefully.** If the command fails, guide the user to install it.
- **Abbreviate session IDs** to 8 characters in display, but use enough chars for uniqueness.
- **For `/ai-history context`**, actually read and internalize the exported content so you
  can reference it in subsequent conversation. Don't just display it — understand it.
