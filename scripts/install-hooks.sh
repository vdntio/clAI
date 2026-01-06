#!/usr/bin/env bash
# Install Git hooks for clai
# Run this script once after cloning the repository

set -e

REPO_ROOT="$(git rev-parse --show-toplevel)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"
SCRIPTS_HOOKS_DIR="$REPO_ROOT/scripts/hooks"

echo "Installing Git hooks for clai..."

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Install pre-commit hook
if [ -f "$SCRIPTS_HOOKS_DIR/pre-commit" ]; then
    echo "Installing pre-commit hook..."
    cp "$SCRIPTS_HOOKS_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
    chmod +x "$HOOKS_DIR/pre-commit"
    echo "✓ Pre-commit hook installed"
else
    echo "✗ Warning: scripts/hooks/pre-commit not found"
fi

echo ""
echo "✓ Git hooks installation complete!"
echo ""
echo "The pre-commit hook will now run automatically before each commit."
echo "It will:"
echo "  1. Format your code with 'cargo fmt'"
echo "  2. Run 'cargo clippy' with warnings as errors"
echo "  3. Run 'cargo test'"
echo ""
echo "To bypass the hook temporarily, use: git commit --no-verify"
