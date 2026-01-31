# Installation

> **Prerequisites**: You'll need an OpenRouter API key from [openrouter.ai](https://openrouter.ai)

## Option 1: npm (Recommended)

Requires Node.js 18+:

```bash
npm install -g clai@alpha
```

Verify installation:
```bash
clai --version
```

## Option 2: Standalone Binary

### Linux / macOS

1. Download the appropriate binary from [Releases](https://github.com/vdntio/clAI/releases):
   - Linux x64: `clai-linux-x64.tar.gz`
   - Linux ARM64: `clai-linux-arm64.tar.gz`
   - macOS x64: `clai-darwin-x64.tar.gz`
   - macOS ARM64 (M1/M2): `clai-darwin-arm64.tar.gz`

2. Extract:
   ```bash
   tar -xzf clai-*.tar.gz
   ```

3. Move to PATH:
   ```bash
   sudo mv clai-* /usr/local/bin/clai
   ```

4. Verify:
   ```bash
   clai --version
   ```

### Windows

1. Download `clai-windows-x64.zip` from [Releases](https://github.com/vdntio/clAI/releases)

2. Extract the `.exe` file to desired location (e.g., `C:\Program Files\clai\`)

3. Add directory to PATH:
   - Press `Win + R`, type `sysdm.cpl`, press Enter
   - Go to "Advanced" tab → "Environment Variables"
   - Under "User variables", select "Path" → "Edit"
   - Click "New" → Add `C:\Program Files\clai`
   - Click "OK" on all dialogs

4. Restart your terminal and verify:
   ```cmd
   clai --version
   ```

## Verification

All binaries are checksummed:
```bash
sha256sum -c SHA256SUMS.txt
```

## Configuration

clAI requires an OpenRouter API key. Set it via:

1. Environment variable:
   ```bash
   export OPENROUTER_API_KEY="your-key-here"
   ```

2. Config file (`.clai.toml` in project or `~/.config/clai/config.toml`):
   ```toml
   openrouter_api_key = "your-key-here"
   ```

## Troubleshooting

### "Command not found: clai"
- Ensure `~/.local/bin` or `/usr/local/bin` is in your PATH
- Run `which clai` to verify installation location

### "Permission denied"
- Ensure binary is executable: `chmod +x /path/to/clai`

### "API key not configured"
- Set `OPENROUTER_API_KEY` environment variable
- Or create config file as shown above

### "Config file has insecure permissions"
- Config files must be readable only by owner
- Fix with: `chmod 600 ~/.config/clai/config.toml`

For more issues, see [GitHub Issues](https://github.com/vdntio/clAI/issues)
