# AGENTS.md

## Project Overview

`ai-history` is a standalone Rust CLI tool that searches and exports AI coding assistant chat history. It reads conversation data from Claude Code, Codex CLI, and Cursor, outputting in Markdown, JSON, Prompt, or Digest format.

Designed to be used inside other AI tool sessions to inject historical context.

## Development Commands

```bash
cargo build            # Debug build
cargo test             # Run unit tests
cargo build --release  # Release build (LTO + strip)
cargo run -- list      # Test with real data
cargo run -- digest <session-id>  # Test digest
```

## Architecture

```
src/
  main.rs              # CLI dispatch (clap)
  cli.rs               # Clap derive subcommand definitions
  model.rs             # Provider-agnostic types: Project, Session, Message, SearchResult
  parse.rs             # JSONL parsing: mmap + simd-json + memchr line splitting
  scoring.rs           # BM25 relevance scoring + tokenizer
  search.rs            # Cross-provider search delegation
  digest/
    mod.rs             # SessionDigest struct, format_digest(), get_or_create_digest()
    extractor.rs       # Rule-based extraction engine (intent, decisions, code changes, issues)
    cache.rs           # Disk cache with mtime+size invalidation
    llm.rs             # Optional Codex API enhancement (--llm flag)
  provider/
    mod.rs             # Provider trait + ProviderRegistry
    claude.rs          # Claude Code: ~/.claude/projects/ JSONL parser
    codex.rs           # Codex CLI: ~/.codex/sessions/ rollout JSONL parser
    cursor.rs          # Cursor: workspaceStorage vscdb SQLite parser
  output/
    mod.rs             # TTY detection
    human.rs           # Colored terminal output (tables, conversation view)
    json.rs            # JSON serialization
    markdown.rs        # Markdown export
    prompt.rs          # Clean User:/Assistant: prompt format
```

## Key Design Decisions

- **Provider trait**: `provider/mod.rs` defines the `Provider` trait. Each provider implements scan/list/load/search independently.
- **Flat Message model**: `Message.text` is pre-flattened from content blocks. Unlike the reference project (Codex-history-viewer) which preserves raw JSON for frontend rendering, this CLI only needs text.
- **Path decoding**: Claude Code encodes project paths with hyphens (`-Users-jack-myapp`). Decoded via filesystem-based recursive lookup in `claude.rs::decode_path_with_prefix()`.
- **Codex CLI deduplication**: Codex rollout JSONL can duplicate user/assistant messages in both `response_item` and `event_msg`. Only `response_item` is used for user/assistant messages.
- **Cursor vscdb**: Cursor stores chat history in SQLite (`state.vscdb`) rather than JSONL. Uses `rusqlite` to query `cursorDiskKV` table, JSON-parses bubble arrays from workspace storage.
- **Pipe detection**: `std::io::IsTerminal` — TTY gets colored output, pipe gets JSON.
- **Session Digest**: Rule-based extraction (zero-cost, offline) compresses sessions to ~5% of original size. Extracts intent from first user message, decisions from thinking blocks, code changes from tool calls, issues from error patterns. Optional `--llm` flag enhances via Anthropic API. Cached to disk with mtime+size invalidation.
- **Context defaults to digest**: `context <id>` outputs digest, `--full` restores original full-text behavior. Saves 10-20x tokens while preserving key information.

## Adding a New Provider

1. Create `src/provider/<name>.rs` implementing `Provider` trait
2. Add `pub mod <name>;` in `src/provider/mod.rs`
3. Register in `build_registry()` in `src/provider/mod.rs`

## Reference Project

This project's parsing logic is informed by [Codex-history-viewer](../Codex-history-viewer/) (Tauri desktop app). Key reference files:
- `src-tauri/src/providers/Codex.rs` — Codex JSONL format
- `src-tauri/src/providers/codex.rs` — Codex rollout format
- `src-tauri/src/utils.rs` — `find_line_ranges`, `decode_project_path`
