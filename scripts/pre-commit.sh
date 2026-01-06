#!/usr/bin/env bash
# Pre-commit checks for clAI
# Run this script before committing to ensure code quality

set -e

echo "Running pre-commit checks..."

echo "1. Formatting code..."
cargo fmt

echo "2. Running clippy..."
cargo clippy -- -D warnings

echo "3. Running tests..."
cargo test

echo "✓ All pre-commit checks passed!"
