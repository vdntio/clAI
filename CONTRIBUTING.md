# Contributing to clai

## Development Setup

### Prerequisites

- Rust 1.70+
- OpenRouter API key (for testing AI features)

```bash
git clone https://github.com/Vedaant-Rajoo/clAI.git
cd clAI
cargo build

# Install Git hooks (recommended)
./scripts/install-hooks.sh
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

| Variable             | Description            |
| -------------------- | ---------------------- |
| `OPENROUTER_API_KEY` | API key for OpenRouter |
| `NO_COLOR`           | Disable colored output |

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

| Code | Meaning                                   |
| ---- | ----------------------------------------- |
| 0    | Success                                   |
| 1    | General error                             |
| 2    | Usage error                               |
| 3    | Configuration error                       |
| 4    | API error                                 |
| 5    | Safety error (dangerous command rejected) |
| 130  | Interrupted (Ctrl+C)                      |

## Pull Request Process

1. Fork and create a feature branch
2. Install Git hooks: `./scripts/install-hooks.sh` (if not already done)
3. Make changes
4. The pre-commit hook will automatically run checks before each commit:
   - Format code with `cargo fmt`
   - Run `cargo clippy -- -D warnings`
   - Run `cargo test`
5. If you need to bypass the hook temporarily: `git commit --no-verify`
6. Submit PR

### Manual Checks

If you haven't installed the Git hooks, run these commands before committing:

```bash
./scripts/pre-commit.sh  # Run all checks
```

Or individually:

```bash
cargo fmt                      # Format code
cargo clippy -- -D warnings    # Check lints
cargo test                     # Run tests
```

## Code Style

- Follow `cargo fmt` formatting
- Use `cargo clippy` for lints
- Write tests for new features
- Document public APIs
