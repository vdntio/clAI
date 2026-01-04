# clAI

AI-powered shell command translator that converts natural language instructions into executable shell commands. Built with Rust for performance, safety, and cross-platform compatibility.

## Features

- 🤖 **AI-Powered**: Uses OpenRouter API to generate shell commands from natural language
- 🔒 **Safety First**: Detects dangerous commands and prompts for confirmation
- 🎯 **Interactive Mode**: Cycle through multiple command options with Tab, execute with Enter
- ⚙️ **Configurable**: XDG-compliant config files with environment variable support
- 🚀 **Fast**: Optimized for <50ms startup time with lazy loading and caching
- 🐚 **Shell-Agnostic**: Works with bash, zsh, fish, and PowerShell
- 📦 **Single Binary**: No runtime dependencies, easy installation

## Prerequisites

### Required

- **Rust** (1.92.0 or newer)
  ```bash
  # Install Rust via rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source $HOME/.cargo/env
  
  # Verify installation
  rustc --version  # Should show 1.92.0 or newer
  cargo --version
  ```

- **OpenRouter API Key** (for AI command generation)
  - Sign up at [OpenRouter.ai](https://openrouter.ai)
  - Get your API key from the dashboard
  - Free tier available with rate limits

### Optional (for development)

- **cargo-make** (for build automation)
  ```bash
  cargo install cargo-make
  ```

- **cargo-edit** (for dependency management)
  ```bash
  cargo install cargo-edit
  ```

## Installation

### From Source

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd clAI
   ```

2. **Build the project**
   ```bash
   # Debug build (faster compilation)
   cargo build

   # Release build (optimized, recommended)
   cargo build --release
   ```

3. **Install globally** (optional)
   ```bash
   # Install to ~/.cargo/bin (or $CARGO_HOME/bin)
   cargo install --path .

   # Or add to PATH manually
   export PATH="$PATH:$(pwd)/target/release"
   ```

### Quick Start

After building, the binary is available at `target/release/clai` (or `target/debug/clai` for debug builds).

## Configuration

### Environment Variables

Set your OpenRouter API key:

```bash
export OPENROUTER_API_KEY="sk-or-v1-your-api-key-here"
```

### Config Files

clAI follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

1. **Project-local**: `./.clai.toml` (highest priority)
2. **User config**: `$XDG_CONFIG_HOME/clai/config.toml` or `~/.config/clai/config.toml`
3. **System config**: `/etc/clai/config.toml` (lowest priority)

**Example config file** (`~/.config/clai/config.toml`):

```toml
[provider]
default = "openrouter"
api-key = "${OPENROUTER_API_KEY}"  # References environment variable

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

**Security Note**: Config files must have `0600` permissions (read/write for owner only) on Unix systems.

## Usage

### Basic Usage

```bash
# Generate a command from natural language
clai "list all files in current directory"

# With multiple options (interactive mode)
clai --options 3 --interactive "find all Python files"

# Dry run (show command without executing)
clai --dry-run "remove old log files"
```

### Command-Line Options

```bash
clai [OPTIONS] <INSTRUCTION>

Arguments:
  <INSTRUCTION>  Natural language instruction to convert to a command

Options:
  -m, --model <MODEL>           Override the AI model to use
  -p, --provider <PROVIDER>     Override the AI provider to use
  -q, --quiet                   Suppress non-essential output
  -v, --verbose...              Increase verbosity (can be used multiple times)
      --no-color                Disable colored output
      --color <COLOR>           Control colored output: auto, always, or never [default: auto]
  -i, --interactive             Interactive mode: prompt for execute/copy/abort
  -f, --force                   Skip dangerous command confirmation
  -n, --dry-run                 Show command without execution prompt
  -c, --context <CONTEXT>       Additional context file
      --offline                 Offline mode (fail gracefully if no local model)
  -o, --options <NUM>           Number of command options to generate [default: 3]
  -h, --help                    Print help
  -V, --version                 Print version
```

### Interactive Mode

When using `--interactive` with multiple options (`--options 3`):

- **Tab**: Cycle through command options (replaces command inline)
- **Enter**: Execute the currently selected command
- **Ctrl+C / Esc**: Abort and exit

Example:
```bash
clai --interactive --options 3 "find large files"
# Shows: [1/3] find / -type f -size +100M
# Press Tab to see: [2/3] find . -type f -size +100M -exec ls -lh {} \;
# Press Enter to execute
```

## Development

### Project Structure

```
clAI/
├── src/
│   ├── main.rs              # Binary entry point
│   ├── lib.rs               # Library entry point
│   ├── cli/                 # CLI argument parsing
│   ├── config/              # Configuration system
│   ├── context/             # Context gathering (system, directory, history)
│   ├── ai/                  # AI provider abstraction
│   │   ├── providers/       # Provider implementations (OpenRouter, etc.)
│   │   └── ...
│   ├── safety/              # Safety checks and dangerous command detection
│   ├── error/               # Error handling and exit codes
│   └── ...
├── tests/                   # Integration tests
├── benches/                 # Performance benchmarks
├── examples/                # Example programs
└── Cargo.toml              # Rust project manifest
```

### Build Commands

Using **Cargo** (standard):
```bash
cargo build              # Debug build
cargo build --release    # Optimized release build
cargo run -- "instruction"  # Build and run
cargo test               # Run tests
cargo clippy             # Lint code
cargo fmt                # Format code
```

Using **Cargo aliases** (from `.cargo/config.toml`):
```bash
cargo b                   # Build (debug)
cargo r -- "instruction" # Run
cargo t                   # Test
cargo cl                  # Clippy
cargo f                   # Format
```

Using **cargo-make** (from `Makefile.toml`):
```bash
cargo make build          # Build release
cargo make run            # Run with example
cargo make test           # Run all tests
cargo make lint           # Run clippy + fmt
cargo make clean           # Clean build artifacts
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test cli_tests
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --features bench

# Run specific benchmark
cargo bench --bench startup --features bench

# Quick test (verify benchmarks compile)
cargo bench --bench startup --features bench -- --test
```

See [BENCHMARKS.md](BENCHMARKS.md) for detailed benchmark documentation.

### Code Quality

```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy -- -D warnings

# Run both (pre-commit check)
cargo make lint
```

### Development Workflow

1. **Make changes** to source code
2. **Test locally**: `cargo test`
3. **Check formatting**: `cargo fmt --check`
4. **Run linter**: `cargo clippy -- -D warnings`
5. **Build release**: `cargo build --release`
6. **Test binary**: `./target/release/clai "test instruction"`

## Configuration Details

### Environment Variables

- `OPENROUTER_API_KEY`: OpenRouter API key (required for AI features)
- `NO_COLOR`: Disable colored output (see [no-color.org](https://no-color.org))
- `CLICOLOR`: Control colored output (0=disable, 1=enable)
- `TERM`: Terminal type (if `dumb`, colors are disabled)

### Config File Format

See the [example config](#config-files) above. Config files support:
- Environment variable references: `${VAR_NAME}` or `$VAR_NAME`
- Multi-level merging (CLI > env > files > defaults)
- Provider-specific settings
- Safety pattern customization
- Context gathering limits

## Exit Codes

Following UNIX conventions:

- `0`: Success
- `1`: General error
- `2`: Usage error (invalid CLI arguments)
- `3`: Configuration error
- `4`: API error (network/auth/rate limit)
- `5`: Safety error (dangerous command rejected)
- `130`: Interrupted (SIGINT)

## Troubleshooting

### Build Issues

**Error: `rustc 1.92.0 or newer required`**
```bash
rustup update stable
```

**Error: `OpenSSL not found`**
- clAI uses `rustls` (no OpenSSL required)
- If you see this error, check your `Cargo.toml` dependencies

### Runtime Issues

**Error: `Failed to get response from AI provider`**
- Check your `OPENROUTER_API_KEY` is set correctly
- Verify API key is valid: `echo $OPENROUTER_API_KEY`
- Check network connectivity

**Error: `Configuration error: ...`**
- Verify config file permissions: `chmod 600 ~/.config/clai/config.toml`
- Check TOML syntax is valid
- See config file paths in order of precedence above

**Command not found after installation**
- Add `~/.cargo/bin` to your PATH:
  ```bash
  export PATH="$PATH:$HOME/.cargo/bin"
  # Add to ~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish
  ```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Check linting: `cargo clippy -- -D warnings`
7. Commit your changes: `git commit -m "Add feature"`
8. Push to branch: `git push origin feature/your-feature`
9. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use `cargo clippy` for linting
- Write tests for new features
- Document public APIs with doc comments
- Follow functional programming paradigms where possible

## License

[Add your license here]

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- AI powered by [OpenRouter](https://openrouter.ai)
- Follows [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/)
