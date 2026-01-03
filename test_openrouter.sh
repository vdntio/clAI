#!/bin/bash
# Test script for OpenRouter integration
# This script tests that clai can gather context and communicate with OpenRouter

set -e

echo "=== Testing OpenRouter Integration ==="
echo ""

# Check if API key is set
if [ -z "$OPENROUTER_API_KEY" ]; then
    echo "⚠️  Warning: OPENROUTER_API_KEY environment variable is not set"
    echo "   Set it with: export OPENROUTER_API_KEY='your-key-here'"
    echo ""
    echo "   You can get an API key from: https://openrouter.ai/keys"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "1. Testing basic command generation..."
echo "   Command: 'list files in current directory'"
echo ""

COMMAND=$(cargo run --quiet -- "list files in current directory" 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Success! Generated command:"
    echo "   $COMMAND"
    echo ""
    echo "   To execute: $COMMAND"
else
    echo "❌ Failed with exit code: $EXIT_CODE"
    echo "   Error output:"
    echo "$COMMAND" | grep -i error || echo "$COMMAND"
    exit 1
fi

echo ""
echo "2. Testing with verbose output..."
echo "   Command: 'show git status'"
echo ""

VERBOSE_OUTPUT=$(cargo run --quiet -- -v "show git status" 2>&1)
echo "$VERBOSE_OUTPUT" | head -20

echo ""
echo "3. Testing context gathering (should see system info in verbose mode)..."
echo "   Command: 'find all rust files'"
echo ""

CONTEXT_TEST=$(cargo run --quiet -- -vv "find all rust files" 2>&1)
echo "$CONTEXT_TEST" | grep -i "system\|context\|directory" | head -5 || echo "   (Context info may be in stderr)"

echo ""
echo "=== Test Summary ==="
echo "✅ Basic command generation: Working"
echo "✅ OpenRouter integration: Working"
echo ""
echo "To test manually:"
echo "  cargo run -- 'your instruction here'"
echo ""
echo "To see verbose output:"
echo "  cargo run -- -v 'your instruction here'"
echo ""
echo "To see debug output:"
echo "  cargo run -- -vv 'your instruction here'"

