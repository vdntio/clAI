# Test Commands for clAI

## Basic Functionality Tests

### 1. Basic Command Execution
```bash
# Simple instruction
cargo r -- "list all files in current directory"

# Check exit code
cargo r -- "test" && echo "Success: Exit code $?"
```

### 2. Help and Version
```bash
# Help output
cargo r -- --help

# Version output
cargo r -- --version
```

### 3. Exit Code Verification
```bash
# Success (should be 0)
cargo r -- "test"; echo "Exit code: $?"

# Invalid arguments (should be 2)
cargo r -- --invalid-flag; echo "Exit code: $?"

# Missing required argument (should be 2)
cargo r --; echo "Exit code: $?"
```

## Color Detection Tests

### 4. Color Detection with Environment Variables
```bash
# Disable colors via NO_COLOR
NO_COLOR=1 cargo r -- --verbose --verbose "test" 2>&1

# Disable colors via TERM=dumb
TERM=dumb cargo r -- --verbose --verbose "test" 2>&1

# Disable colors via --no-color flag
cargo r -- --no-color --verbose --verbose "test" 2>&1

# Compare with colors enabled (default)
cargo r -- --verbose --verbose "test" 2>&1
```

## Logging and Verbosity Tests

### 5. Verbosity Levels
```bash
# Default (Warning level - no debug output)
cargo r -- "test" 2>&1

# Verbose level 1 (Info)
cargo r -- --verbose "test" 2>&1

# Verbose level 2 (Debug)
cargo r -- --verbose --verbose "test" 2>&1

# Verbose level 3 (Trace)
cargo r -- --verbose --verbose --verbose "test" 2>&1
```

### 6. Quiet Mode
```bash
# Quiet mode (errors only)
cargo r -- --quiet "test" 2>&1

# Compare with default
cargo r -- "test" 2>&1
```

## Stdout/Stderr Separation Tests

### 7. Pipe Compatibility
```bash
# Stdout should be clean (only command output)
cargo r -- "test" 2>/dev/null

# Stderr should contain logs
cargo r -- --verbose --verbose "test" 2>&1 >/dev/null

# Pipe to another command
cargo r -- "test" 2>/dev/null | wc -w

# Should output exactly 6 words: "Command would be generated for: test"
cargo r -- "test" 2>/dev/null | wc -w
```

### 8. Verify Clean Stdout
```bash
# Count words in stdout (should be 6: "Command would be generated for: test")
cargo r -- "test" 2>/dev/null | wc -w

# Verify no logs in stdout
cargo r -- --verbose --verbose "test" 2>/dev/null | grep -v "Command would be generated" || echo "Stdout is clean!"
```

## TTY Detection Tests

### 9. TTY Detection (Interactive vs Piped)
```bash
# Interactive mode (TTY)
cargo r -- "test" 2>&1

# Piped mode (not TTY)
echo "test" | cargo r -- "list files" 2>&1

# Redirected output (not TTY)
cargo r -- "test" > output.txt 2>&1 && cat output.txt
```

## CLI Flag Tests

### 10. All CLI Flags
```bash
# Model flag
cargo r -- --model "gpt-4" "test instruction"

# Provider flag
cargo r -- --provider "openai" "test instruction"

# Interactive flag
cargo r -- --interactive "test instruction"

# Force flag
cargo r -- --force "test instruction"

# Dry run flag
cargo r -- --dry-run "test instruction"

# Context flag
cargo r -- --context "current directory" "list files"

# Offline flag
cargo r -- --offline "test instruction"

# Multiple flags combined
cargo r -- --verbose --no-color --quiet "test" 2>&1
```

## Signal Handling Tests (Manual)

### 11. Signal Handling
```bash
# Start the program and press Ctrl+C
# Should exit with code 130
cargo r -- "test" &
PID=$!
sleep 1
kill -INT $PID
wait $PID
echo "Exit code: $?"

# SIGTERM test
cargo r -- "test" &
PID=$!
sleep 1
kill -TERM $PID
wait $PID
echo "Exit code: $?"
```

## Integration Tests

### 12. Real-world Usage Scenarios
```bash
# Simulate piping to another command
cargo r -- "list python files" 2>/dev/null | head -1

# Chain with other commands
cargo r -- "count lines" 2>/dev/null | wc -l

# Use in a script
cargo r -- "test" 2>/dev/null > /tmp/output.txt && cat /tmp/output.txt
```

## Test Suite Summary

Run this comprehensive test:
```bash
echo "=== Basic Test ==="
cargo r -- "test" && echo "✓ Basic works"

echo "=== Exit Code Test ==="
cargo r -- "test"; [ $? -eq 0 ] && echo "✓ Exit code 0"
cargo r -- --invalid 2>/dev/null; [ $? -eq 2 ] && echo "✓ Exit code 2"

echo "=== Stdout Clean Test ==="
OUTPUT=$(cargo r -- "test" 2>/dev/null)
[ "$OUTPUT" = "Command would be generated for: test" ] && echo "✓ Stdout clean"

echo "=== Pipe Test ==="
cargo r -- "test" 2>/dev/null | grep -q "Command would be generated" && echo "✓ Pipe works"

echo "=== Color Test ==="
NO_COLOR=1 cargo r -- --verbose --verbose "test" 2>&1 | grep -q "DEBUG" && echo "✓ NO_COLOR works"

echo "=== Verbosity Test ==="
cargo r -- --verbose --verbose "test" 2>&1 | grep -q "DEBUG" && echo "✓ Verbosity works"

echo "All tests completed!"
```

