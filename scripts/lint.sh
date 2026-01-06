#!/usr/bin/env bash
# Linting checks for clai
# Runs clippy and format checks without making changes

set -e

echo "Running lint checks..."

echo "1. Checking code formatting..."
cargo fmt -- --check

echo "2. Running clippy..."
cargo clippy -- -D warnings

echo "✓ All lint checks passed!"
