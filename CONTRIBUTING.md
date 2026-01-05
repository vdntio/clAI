# Contributing to clAI

## Development Setup

### Prerequisites

- Rust 1.70+
- OpenRouter API key (for testing AI features)

```bash
git clone https://github.com/yourusername/clAI.git
cd clAI
cargo build
```

### Running

```bash
cargo run -- "your instruction"
```

## Project Structure

```
src/
├── main.rs          # Entry point
├── lib.rs           # Library exports
├── cli/             # Argument parsing
├── config/          # Configuration loading
├── context/         # System/directory context gathering
├── ai/              # AI provider abstraction
│   └── providers/   # OpenRouter, etc.
├── safety/          # Dangerous command detection
└── error/           # Error types and exit codes
```

## Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run tests
cargo clippy             # Lint
cargo fmt                # Format
cargo bench --features bench  # Run benchmarks
```

## Configuration

### Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENROUTER_API_KEY` | API key for OpenRouter |
| `NO_COLOR` | Disable colored output |

### Config File Locations

1. `./.clai.toml` (project-local, highest priority)
2. `~/.config/clai/config.toml` (user)
3. `/etc/clai/config.toml` (system)

### Full Config Example

```toml
[provider]
default = "openrouter"
api-key = "${OPENROUTER_API_KEY}"

[provider.openrouter]
model = "qwen/qwen3-coder"

[context]
max-history = 3
max-files = 10

[safety]
confirm-dangerous = true
dangerous-patterns = [
  "rm -rf",
  "sudo.*rm",
  ".*> /dev/sd[a-z]",
]

[ui]
interactive = true
color = "auto"
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Usage error |
| 3 | Configuration error |
| 4 | API error |
| 5 | Safety error (dangerous command rejected) |
| 130 | Interrupted (Ctrl+C) |

## Pull Request Process

1. Fork and create a feature branch
2. Make changes
3. Ensure tests pass: `cargo test`
4. Format code: `cargo fmt`
5. Check lints: `cargo clippy -- -D warnings`
6. Submit PR

## Code Style

- Follow `cargo fmt` formatting
- Use `cargo clippy` for lints
- Write tests for new features
- Document public APIs
