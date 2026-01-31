# clAI

AI-powered CLI that converts natural language instructions into executable shell commands using OpenRouter.

## Example

```bash
clai "find all TypeScript files modified in the last 7 days"
# Outputs: find . -name "*.ts" -mtime -7

clai "list top 10 largest files in current directory"
# Outputs: du -h * | sort -rh | head -10
```

## Features

- 🤖 Natural language to shell commands using AI
- 🎨 Beautiful terminal UI with Ink
- 🔒 Safety checks for dangerous commands (rm -rf, etc.)
- 📝 Context-aware (current directory, shell type, git status)
- ⚡ Fast startup with Bun runtime
- 🌍 Cross-platform (Linux, macOS, Windows)

## Installation

### npm (Recommended)

```bash
npm install -g clai@alpha
```

### Standalone Binary

Download from [Releases](https://github.com/vdntio/clAI/releases)

See [Installation Guide](docs/installation.md) for detailed instructions.

## Quick Start

1. Get an OpenRouter API key from [openrouter.ai](https://openrouter.ai)

2. Set your API key:
   ```bash
   export OPENROUTER_API_KEY="your-key-here"
   ```

3. Run a command:
   ```bash
   clai "your natural language instruction"
   ```

## Configuration

clAI uses TOML config files. Priority order:
1. `./.clai.toml` (project-level)
2. `~/.config/clai/config.toml` (user-level)
3. `/etc/clai/config.toml` (system-level)

Example config:
```toml
openrouter_api_key = "sk-..."
model = "anthropic/claude-3.5-sonnet"
safety_checks = true
```

## Development

```bash
# Install dependencies
bun install

# Run in development mode
bun run dev

# Run tests
bun test

# Build
bun run build
```

## License

MIT

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.