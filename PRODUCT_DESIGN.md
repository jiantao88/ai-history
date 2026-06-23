# ai-history Mac App Product Design

## Product Positioning

`ai-history for Mac` is a local-first AI coding memory workspace. It turns Claude Code, Codex CLI, and Cursor chat history into searchable, reusable, and explainable context.

The product should feel like a professional macOS developer tool: dense, calm, dark, direct, and optimized for repeated daily use.

## Core User Jobs

1. Find prior AI coding conversations quickly.
2. Understand what happened in a session without reading the full transcript.
3. Convert past sessions into clean context for a new AI coding run.
4. Review today's AI coding work across tools.
5. Identify repeated manual workflows worth packaging as skills or automation.
6. Diagnose missing or stale provider history.

## Visual Direction

Reference style: dark agent dashboard.

- Background: near-black app shell.
- Panels: dark elevated cards with subtle borders.
- Accent: purple-blue active states.
- Supporting status colors: green for connected/complete, yellow for waiting, red for error, blue for secondary data.
- Layout: fixed left navigation, top utility bar, dense cards, status pills, operational panels.
- Corners: 8px for cards and controls, pill radius only for badges.
- Typography: SF Pro-style UI font, SF Mono for paths and code-like labels.

## Information Architecture

### Dashboard

Purpose: show local history health and recent work.

Primary content:

- Total Sessions
- Projects
- Messages
- Today Work
- Digest Cache
- Privacy status
- Recent Sessions
- Today Work activity

Key interactions:

- Click a session to open session detail.
- Click Today Work to enter Worklog.
- Refresh rescans local sources.
- Provider filter narrows dashboard data.

### Search

Purpose: BM25-style cross-provider history search.

Layout:

- Left filters: query, search mode, role filters, context window.
- Center results: score, title, provider, project, excerpt.
- Right preview: transcript/digest/files/tools tabs.

Key interactions:

- Top search focuses Search page.
- Empty query shows all sessions.
- Any terms / All terms changes matching.
- Result selection updates preview.
- Copy Digest copies generated digest context.

### Sessions

Purpose: browse projects and inspect sessions.

Layout:

- Left project list.
- Center session list.
- Right session detail.

Session detail tabs:

- Transcript
- Digest
- Files
- Tools

Key interactions:

- Export sessions.
- Switch tabs.
- Generate or copy digest.
- Inspect files and tools.

### Context Builder

Purpose: build reusable context packs from one or more sessions.

Layout:

- Sources list with selectable sessions.
- Context Pack editor with selected sessions.
- Output panel with token estimate and output actions.

Modes:

- Digest
- Prompt
- Full

Key interactions:

- Select/deselect sessions.
- Switch mode and update token estimate.
- Copy context.
- Export Markdown.

### Worklog

Purpose: summarize AI coding activity by date/project/provider.

Tabs:

- Today
- Summary

Key interactions:

- Copy today's titles or summary.
- View session metadata.
- Preserve uncertainty: activity summaries represent AI session activity, not definitive human work effort.

### Workflows

Purpose: identify repeated manual workflows worth packaging.

Content:

- Candidate list
- Confidence
- Frequency
- Recommendation
- Coverage
- Evidence sessions

Key interactions:

- Select candidate.
- Preview skill draft.
- Write skill only when candidate is marked worth creating.

### Providers

Purpose: explain what data sources are connected and why history may be missing.

Provider cards:

- Claude Code
- Codex CLI
- Cursor

Key interactions:

- Rescan provider.
- Open diagnostics.
- Copy diagnostics.

### Settings

Purpose: configure app behavior.

Groups:

- Privacy
- LLM Enhancement
- Cache
- Exports

Key interactions:

- Toggle settings with immediate feedback.
- Test LLM configuration in future implementation.
- Clear/rebuild cache in future implementation.

## Global Interactions

- Default language: Chinese.
- Language switcher belongs in the top-right toolbar, not the sidebar.
- `Cmd+K` opens the command palette.
- Global search moves to Search and filters results.
- Toasts confirm background actions.
- Refresh indicates a local rescan, not a cloud sync.

## Prototype Scope

The first prototype is frontend-only with realistic mock data. It should demonstrate:

- Navigation across all major pages.
- Search filtering.
- Session preview and tabs.
- Context pack generation.
- Provider diagnostics modal.
- Workflow skill draft modal.
- Settings toggle feedback.
- Chinese/English UI switching.

## Future App Architecture Notes

Recommended implementation path:

1. Keep the Rust CLI as the local data and parsing engine.
2. Expose a local app-facing API or Tauri command layer.
3. Build the Mac app as a Tauri desktop shell.
4. Reuse provider registry, digest, summary, today, and workflows logic from the CLI.
5. Store UI preferences separately from provider data.

The app must remain local-first. Optional LLM calls should require explicit user action.
