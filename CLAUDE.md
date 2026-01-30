# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

clai is a CLI tool that converts natural language into shell commands using AI. It follows Unix philosophy: simple, composable, and privacy-respecting. Users provide instructions like `clai "find all rust files modified today"` and get executable shell commands back.

## Build and Development Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run all tests
cargo test <test_name>   # Run a single test
cargo clippy -- -D warnings  # Lint (warnings are errors)
cargo fmt                # Format code
cargo bench --features bench  # Run benchmarks
cargo run -- "instruction"   # Run with an instruction
```

Pre-commit checks (run before committing):
```bash
./scripts/pre-commit.sh  # Runs fmt, clippy, and tests
```

## Architecture

### Core Flow

`main.rs` orchestrates the pipeline:
1. Parse CLI args → `cli/mod.rs` (clap-based)
2. Load config → `config/` (layered: .clai.toml > ~/.config/clai/config.toml > /etc/clai/config.toml)
3. Gather context → `context/` (system info, cwd, files, shell history, stdin)
4. Build prompt → `ai/prompt.rs`
5. Call AI provider → `ai/providers/` (OpenRouter via provider chain)
6. Safety check → `safety/` (dangerous command detection with regex patterns)
7. Output or execute → `output/`, `safety/interactive.rs`

### Key Modules

- **ai/**: AI provider abstraction with async traits. `handler.rs` orchestrates generation, `chain.rs` manages provider fallback, `providers/openrouter.rs` is the primary implementation.
- **config/**: Layered config system. `paths.rs` discovers config files, `loader.rs` loads/validates TOML, `merger.rs` combines configs, `cache.rs` provides lazy caching.
- **context/**: Gathers system context for prompts. `gatherer.rs` orchestrates, individual modules handle system info, directory scanning, shell history, and stdin.
- **safety/**: Dangerous command detection. `patterns.rs` defines regex patterns, `detector.rs` matches commands, `confirmation.rs` and `interactive.rs` handle user prompts.
- **error/**: Typed errors with exit codes (1=general, 2=usage, 3=config, 4=API, 5=safety, 130=interrupted).

### Design Patterns

- **Pure functions preferred**: Most functions are pure transformations. I/O is isolated to main.rs and handler functions.
- **Immutable Config**: Config struct is immutable after creation from CLI args.
- **Strict stdout/stderr**: stdout = commands only (for piping), stderr = logs/warnings.
- **Async with tokio**: AI calls use async/await with reqwest.

## Configuration

Environment variables:
- `OPENROUTER_API_KEY` - Required for AI provider
- `NO_COLOR` - Disable colored output

Config file locations (highest to lowest priority):
1. `./.clai.toml` (project-local)
2. `~/.config/clai/config.toml` (user)
3. `/etc/clai/config.toml` (system)

## Testing

Tests are co-located in the same files as implementation (`#[cfg(test)]` modules). Run specific test file with:
```bash
cargo test --lib config::tests
cargo test --lib safety::tests
```
