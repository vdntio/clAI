# clAI

A CLI tool that converts natural language into shell commands using AI.

```bash
$ clai "find all rust files modified today"
find . -name "*.rs" -mtime 0
```

## Installation

Requires Rust 1.70+.

```bash
git clone https://github.com/yourusername/clAI.git
cd clAI
cargo install --path .
```

## Setup

1. Get an API key from [OpenRouter](https://openrouter.ai)
2. Set the environment variable:
   ```bash
   export OPENROUTER_API_KEY="your-key-here"
   ```

## Usage

```bash
clai "list files by size"
clai -i "delete old logs"        # interactive mode - confirm before executing
clai -n "dangerous command"      # dry-run - show without executing
clai -o 3 "compress images"      # generate 3 options to choose from
```

### Options

| Flag | Description |
|------|-------------|
| `-i, --interactive` | Prompt before executing |
| `-n, --dry-run` | Show command without executing |
| `-o, --options <N>` | Generate N command options |
| `-f, --force` | Skip safety confirmations |
| `-q, --quiet` | Minimal output |
| `-v, --verbose` | Increase verbosity |

## Configuration

Create `~/.config/clai/config.toml`:

```toml
[provider]
default = "openrouter"

[provider.openrouter]
model = "qwen/qwen3-coder"

[safety]
confirm-dangerous = true
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and full configuration options.

## License

MIT
