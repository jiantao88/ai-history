# CLAUDE.md

## Project Overview

`ai-history` is a standalone Rust CLI tool that searches and exports AI coding assistant chat history. It reads JSONL conversation data from Claude Code and Codex CLI, outputting in Markdown, JSON, or Prompt format.

Designed to be used inside other AI tool sessions to inject historical context.

## Development Commands

```bash
cargo build            # Debug build
cargo test             # Run unit tests (7 tests in parse module)
cargo build --release  # Release build (LTO + strip)
cargo run -- list      # Test with real data
```

## Architecture

```
src/
  main.rs              # CLI dispatch (clap)
  cli.rs               # Clap derive subcommand definitions
  model.rs             # Provider-agnostic types: Project, Session, Message, SearchResult
  parse.rs             # JSONL parsing: mmap + simd-json + memchr line splitting
  search.rs            # Cross-provider search delegation
  provider/
    mod.rs             # Provider trait + ProviderRegistry
    claude.rs          # Claude Code: ~/.claude/projects/ JSONL parser
    codex.rs           # Codex CLI: ~/.codex/sessions/ rollout JSONL parser
  output/
    mod.rs             # TTY detection
    human.rs           # Colored terminal output (tables, conversation view)
    json.rs            # JSON serialization
    markdown.rs        # Markdown export
    prompt.rs          # Clean User:/Assistant: prompt format
```

## Key Design Decisions

- **Provider trait**: `provider/mod.rs` defines the `Provider` trait. Each provider implements scan/list/load/search independently.
- **Flat Message model**: `Message.text` is pre-flattened from content blocks. Unlike the reference project (claude-code-history-viewer) which preserves raw JSON for frontend rendering, this CLI only needs text.
- **Path decoding**: Claude encodes project paths with hyphens (`-Users-jack-myapp`). Decoded via filesystem-based recursive lookup in `claude.rs::decode_path_with_prefix()`.
- **Codex deduplication**: Codex JSONL has duplicate messages in both `response_item` and `event_msg`. Only `response_item` is used for user/assistant messages.
- **Pipe detection**: `std::io::IsTerminal` — TTY gets colored output, pipe gets JSON.

## Adding a New Provider

1. Create `src/provider/<name>.rs` implementing `Provider` trait
2. Add `pub mod <name>;` in `src/provider/mod.rs`
3. Register in `build_registry()` in `src/provider/mod.rs`

## Reference Project

This project's parsing logic is informed by [claude-code-history-viewer](../claude-code-history-viewer/) (Tauri desktop app). Key reference files:
- `src-tauri/src/providers/claude.rs` — Claude JSONL format
- `src-tauri/src/providers/codex.rs` — Codex rollout format
- `src-tauri/src/utils.rs` — `find_line_ranges`, `decode_project_path`
