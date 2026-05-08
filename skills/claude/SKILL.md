---
name: ai-history
version: 0.1.0
description: |
  Search and browse AI coding assistant chat history across providers (Claude Code, Codex CLI).
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
(Claude Code, Codex CLI). It uses the `ai-history` CLI tool installed at `~/.cargo/bin/ai-history`.

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

Parse the JSON output and present as a readable table:

```
AI CHAT HISTORY — PROJECTS
════════════════════════════════════════════════════════════════
#   Provider  Project                          Sessions  Last Active
──  ────────  ───────────────────────────────  ────────  ───────────
1   claude    ~/projects/myapp                 12        2026-05-08
2   codex     ~/projects/myapp                 8         2026-05-07
════════════════════════════════════════════════════════════════
```

### `/ai-history sessions <project>`

List sessions for a project. The `<project>` argument supports fuzzy matching (substring).

```bash
~/.cargo/bin/ai-history sessions "<project>" --json 2>/dev/null
```

Present as a table with session ID abbreviated to first 8 chars.

### `/ai-history show <session-id>`

Show messages from a specific session. Session ID can be abbreviated (first 8+ chars).

```bash
~/.cargo/bin/ai-history show "<session-id>" --compact --json 2>/dev/null
```

Display the conversation in a readable format. If very long (>50 messages), show
first 10 and last 10 with a summary in between.

### `/ai-history search <query>`

Search across all chat history for a keyword or phrase.

```bash
~/.cargo/bin/ai-history search "<query>" --limit 10 --json 2>/dev/null
```

Truncate each text snippet to ~200 chars.

### `/ai-history context <session-id>`

Export a session in prompt format and inject it as context into the current conversation.

```bash
~/.cargo/bin/ai-history export "<session-id>" --format prompt 2>/dev/null
```

After fetching:
1. Read and understand the exported conversation
2. Present a brief summary:
   ```
   CONTEXT LOADED
   ════════════════════════════════════════
   Session:  <summary>
   Provider: <provider>
   Project:  <project>
   Date:     <date>
   Messages: <count> (User/Assistant only)
   ════════════════════════════════════════
   ```
3. Tell the user you now have the context and can continue the work.

### `/ai-history context-search <query>`

Shortcut: search then automatically load the most relevant session as context.

1. Run search: `~/.cargo/bin/ai-history search "<query>" --limit 5 --json 2>/dev/null`
2. If one clear match, load it directly
3. If multiple, ask the user which session to load

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
