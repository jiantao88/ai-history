# Design QA

final result: screenshot-qa-partial

## Scope

Prototype: ai-history Mac app high-fidelity dashboard prototype.

Reference: user-provided Agent Dashboard screenshots with dark sidebar, purple active states, dense metric cards, status pills, and multi-column operational panels.

## Completed Checks

- Build check passed with `npm run build`.
- Local Vite server started at `http://127.0.0.1:5173/`.
- Service health check returned `HTTP/1.1 200 OK`.
- Implemented interactive navigation for Dashboard, Search, Sessions, Context Builder, Worklog, Workflows, Providers, and Settings.
- Implemented provider filtering, search input behavior, session selection, digest tabs, context pack selection, worklog tabs, workflow candidate selection, and skill preview modal.
- User-provided dashboard screenshot reviewed after moving the language switcher.
- Fixed sidebar bottom compression so `Collapse` and provider status rows keep stable height.
- Fixed `Today Work` activity rows so the main event text stays on one line with ellipsis instead of wrapping into narrow stacked text.
- Kept the language switcher in the top-right toolbar with provider filters and refresh.
- Added functional global search: typing in the top search field moves into Search and filters results.
- Added `⌘K` command palette with navigation commands and search handoff.
- Added toast feedback for refresh, copy digest, context copy, export, LLM enhancement preview, provider rescan, diagnostics copy, settings toggles, and workflow skill writing.
- Added Provider diagnostics modal with per-provider scan notes.
- Added Context Builder output generation across Digest, Prompt, and Full modes.
- Added Workflow skill draft confirmation flow.
- Added real Chinese/English UI switching for the main app chrome, navigation, page titles, filters, tabs, actions, settings, and command palette.
- Changed default UI language to Chinese.
- Added root-level `PRODUCT_DESIGN.md` as the product/design source of truth for future Mac app development.
- Fixed Chinese keyword search in the prototype sample dataset. Terms like `总结`, `工作流`, `缓存`, and `认证` now expand to matching English sample fields without breaking `all terms` mode.
- Fixed Search empty-state behavior so no-result searches do not keep showing a stale session preview.
- Added a visible frontend prototype notice that clarifies current search/export/diagnostics/workflow actions are sample-data interactions, not the real Rust CLI backend yet.
- Fixed provider-filtered Sessions preview so the right-side detail follows the visible provider result set instead of keeping a hidden previous selection.
- Connected the Search page to real local history in dev mode through `/api/search`, backed by `~/.cargo/bin/ai-history search --json`.
- Added real-search loading and error states, provider filtering, `any/all` mode forwarding, and context-window previews from CLI results.
- Confirmed real local search returns Codex and Claude history for the Chinese query `总结`.
- Removed runtime mock data usage from the main app flow.
- Connected Dashboard, Sessions, Providers, Context Builder, and Worklog to `/api/dashboard`, backed by real `ai-history list`, `ai-history sessions`, and `ai-history today` CLI output.
- Connected Workflows to `/api/workflows`, backed by real `ai-history workflows --json` output.
- Verified the Codex dashboard endpoint returns real local counts: 12 projects, 175 sessions, and 3 today work entries.
- Verified the real workflows endpoint returns scanned candidates from local Codex history.
- Fixed Context Builder layout breakage caused by real local history titles and summaries being much longer than the original mock data.
- Added server-side display text cleanup so AGENTS.md/system instruction scaffolding is not used as session card titles.
- Constrained source rows, context pack cards, preview text, and panel overflow so long real paths or summaries cannot cover adjacent columns.

## Remaining Limitation

The current tool context still does not expose a Browser or Chrome screenshot tool for direct capture. The QA above is based on the screenshot supplied by the user in the browser comment thread. A tool-driven same-viewport reference/prototype comparison remains pending unless Playwright use is approved.

## Visual Follow-Up Checklist

- Capture Dashboard at desktop viewport and compare spacing, sidebar density, card sizing, text hierarchy, and purple active states against the supplied reference.
- Capture Search and Sessions views to check three-column density and no text overflow.
- Capture Context Builder and Workflows modal states.
- Check responsive layout at tablet/mobile widths for readable navigation and non-overlapping cards.
