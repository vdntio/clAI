# Installation

## Option 1: npm (Recommended for Developers)

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

1. Download the appropriate binary from [Releases](https://github.com/vdntio/clAI/releases)
2. Extract:
   ```bash
   tar -xzf clai-*-{os}-{arch}.tar.gz
   ```
3. Make executable:
   ```bash
   chmod +x clai
   ```
4. Move to PATH:
   ```bash
   sudo mv clai /usr/local/bin/
   ```
5. Verify:
   ```bash
   clai --version
   ```

### Windows

1. Download `clai-*-windows-x64.zip` from [Releases](https://github.com/vdntio/clAI/releases)
2. Extract to desired location (e.g., `C:\Program Files\clai\`)
3. Add directory to PATH:
   - Search "Environment Variables"
   - Edit "Path" variable
   - Add `C:\Program Files\clai`
4. Verify:
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

For more issues, see [GitHub Issues](https://github.com/vdntio/clAI/issues)
