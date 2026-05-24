# ai-history Demo Script

Use this script to record the short terminal demo for GitHub, Reddit, X, and the launch write-up.

## Goal

Show the product's aha moment in under 20 seconds:

1. A developer searches yesterday's AI coding session.
2. `ai-history` finds the relevant Claude Code, Codex CLI, or Cursor conversation.
3. The developer turns it into a compact digest.
4. The digest becomes reusable context for a new AI session.

## Recommended format

- Aspect ratio: 16:9 for GitHub and articles, 1:1 crop for X.
- Length: 15-20 seconds.
- Style: terminal-first, no narration required.
- Output: `assets/ai-history-demo.gif` and `assets/ai-history-demo.mp4`.

## Terminal setup

Use a clean terminal with a large font and a narrow prompt:

```bash
export CLICOLOR=1
clear
```

Recommended terminal size: 100 columns by 28 rows.

## Shot list

### 1. Problem setup

On screen text:

```text
New AI coding session.
Same project context.
Again.
```

Duration: 2 seconds.

### 2. Search history

Command:

```bash
ai-history search "auth bug" -n 3
```

Expected visual:

```text
1. codex   myapp   Fix OAuth callback regression
2. cursor  myapp   Investigate stale token cache
3. claude  myapp   Refactor auth middleware
```

Duration: 4 seconds.

### 3. Generate reusable context

Command:

```bash
ai-history context <session-id>
```

Expected visual:

```markdown
# Session Digest: Fix OAuth callback regression

## Intent
Fix an OAuth redirect loop after the callback route changed.

## Key Decisions
- Keep callback validation in `auth/callback.ts`.
- Do not move token refresh into middleware.

## Code Changes
- Updated redirect URI handling.
- Added regression coverage for stale token cache.
```

Duration: 7 seconds.

### 4. Close with value proposition

On screen text:

```text
Paste the digest into Claude Code, Codex, Cursor, or any AI session.

Stop re-explaining context.
```

Duration: 3 seconds.

## Exact social caption

```text
ai-history is a local memory layer for AI coding sessions.

Search Claude Code, Codex CLI, and Cursor history, then turn past sessions into compact context for the next assistant.
```

## Recording checklist

- [ ] Terminal output fits without wrapping awkwardly.
- [ ] No private project names, tokens, file paths, customer names, or credentials appear.
- [ ] The first command is visible within the first 5 seconds.
- [ ] The final frame includes `github.com/jiantao88/ai-history`.
- [ ] Export both GIF and MP4.
- [ ] Add the GIF to the README only after the file exists in the repo.
