# OpenRouter Integration Test Guide

This guide helps you test that clai is properly communicating with OpenRouter and receiving context.

## Prerequisites

1. **OpenRouter API Key**: Get one from https://openrouter.ai/keys
2. **Set Environment Variable**:
   ```bash
   export OPENROUTER_API_KEY='your-key-here'
   ```

## Quick Test

Run the automated test script:
```bash
./test_openrouter.sh
```

## Manual Testing

### 1. Basic Command Generation

Test that clai can generate a simple command:
```bash
cargo run -- "list files in current directory"
```

Expected output: A shell command (e.g., `ls -la`) printed to stdout.

### 2. Verbose Mode (See Context)

See what context is being sent to OpenRouter:
```bash
cargo run -- -v "find all rust files"
```

This will show:
- System information being gathered
- Directory context
- Shell history
- The prompt being sent to OpenRouter

### 3. Debug Mode (Maximum Detail)

See all debug information:
```bash
cargo run -- -vv "show git status"
```

### 4. Test with Different Instructions

Try various natural language instructions:
```bash
cargo run -- "count lines in all python files"
cargo run -- "show me the last 10 git commits"
cargo run -- "find files larger than 1MB"
```

## Verifying Context is Sent

The context includes:
- **System Info**: OS, architecture, shell, user
- **Directory Context**: Current directory, file list
- **Shell History**: Recent commands (last 3 by default)
- **Stdin**: If piped input is provided

You can verify this is working by:
1. Running with `-vv` flag to see all context
2. Checking that the generated command is relevant to your current directory
3. Observing that the command considers your shell history

## Testing Model Selection

The default model is `moonshot/kimi-v2` (KimiK2). You can override it:

```bash
# Use a different model
cargo run -- --model "openai/gpt-4" "your instruction"

# Use provider/model format
cargo run -- --model "openrouter/moonshot/kimi-v2" "your instruction"
```

## Expected Behavior

✅ **Success Indicators:**
- Command is generated and printed to stdout
- Command is relevant to your instruction
- Command considers your current directory context
- Exit code is 0

❌ **Failure Indicators:**
- Error message printed to stderr
- Exit code is non-zero
- "API key not found" error
- "Failed to get response from AI provider" error

## Troubleshooting

### "OpenRouter API key not found"
- Ensure `OPENROUTER_API_KEY` is set: `echo $OPENROUTER_API_KEY`
- Or set it in config file: `~/.config/clai/config.toml`

### "Failed to get response from AI provider"
- Check your internet connection
- Verify API key is valid
- Check OpenRouter status: https://status.openrouter.ai/
- Try with verbose flag to see detailed error

### Command seems generic/not context-aware
- Run with `-vv` to verify context is being gathered
- Check that you're in a directory with files
- Verify shell history is being read (check `$HISTFILE`)

## Next Steps

After verifying OpenRouter integration works:
1. Test with different providers (when implemented)
2. Test offline mode (when local providers are added)
3. Test with piped stdin input
4. Test with different shell histories

