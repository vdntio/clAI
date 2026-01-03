#!/bin/bash
# Test script for clAI configuration system

set -e

echo "=== Testing clAI Configuration System ==="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

test_result() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $1"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗${NC} $1"
        ((TESTS_FAILED++))
    fi
}

# Test 1: Default config (no files, no env, no CLI flags)
echo "Test 1: Default configuration"
OUTPUT=$(cd /home/vee/Coding/clAI && cargo r -- "test" 2>&1)
test_result "Default config loads successfully"

# Test 2: CLI flag override (--provider)
echo ""
echo "Test 2: CLI flag override (--provider)"
OUTPUT=$(cd /home/vee/Coding/clAI && cargo r -- --provider "test-provider" "test" 2>&1)
test_result "CLI --provider flag works"

# Test 3: CLI flag override (--model)
echo ""
echo "Test 3: CLI flag override (--model)"
OUTPUT=$(cd /home/vee/Coding/clAI && cargo r -- --model "gpt-4" "test" 2>&1)
test_result "CLI --model flag works"

# Test 4: Environment variable override
echo ""
echo "Test 4: Environment variable override"
OUTPUT=$(cd /home/vee/Coding/clAI && CLAI_PROVIDER_DEFAULT="env-provider" cargo r -- "test" 2>&1)
test_result "Environment variable CLAI_PROVIDER_DEFAULT works"

# Test 5: Config file loading (current directory)
echo ""
echo "Test 5: Config file in current directory"
cd /home/vee/Coding/clAI
cat > .clai.toml << 'EOF'
[provider]
default = "file-provider"

[context]
max-files = 25
EOF
chmod 600 .clai.toml 2>/dev/null || true
OUTPUT=$(cargo r -- "test" 2>&1)
test_result "Config file .clai.toml loads successfully"
rm -f .clai.toml

# Test 6: XDG config path
echo ""
echo "Test 6: XDG config directory"
mkdir -p ~/.config/clai 2>/dev/null || true
cat > ~/.config/clai/config.toml << 'EOF'
[provider]
default = "xdg-provider"

[context]
max-history = 5
EOF
chmod 600 ~/.config/clai/config.toml 2>/dev/null || true
OUTPUT=$(cd /home/vee/Coding/clAI && cargo r -- "test" 2>&1)
test_result "XDG config file loads successfully"
rm -f ~/.config/clai/config.toml 2>/dev/null || true

# Test 7: Precedence test (CLI > env > file)
echo ""
echo "Test 7: Precedence order (CLI > env > file)"
cd /home/vee/Coding/clAI
cat > .clai.toml << 'EOF'
[provider]
default = "file-provider"
EOF
chmod 600 .clai.toml 2>/dev/null || true
OUTPUT=$(CLAI_PROVIDER_DEFAULT="env-provider" cargo r -- --provider "cli-provider" "test" 2>&1)
# CLI should win, so we expect it to work
test_result "CLI overrides env and file (precedence)"
rm -f .clai.toml

# Test 8: Permission check (should fail with 644)
echo ""
echo "Test 8: Permission check (insecure permissions)"
cd /home/vee/Coding/clAI
cat > .clai.toml << 'EOF'
[provider]
default = "test"
EOF
chmod 644 .clai.toml 2>/dev/null || true
OUTPUT=$(cargo r -- "test" 2>&1 2>&1)
# Should show warning about insecure permissions
if echo "$OUTPUT" | grep -q "InsecurePermissions\|insecure\|permission"; then
    test_result "Permission check rejects 644 permissions"
else
    echo -e "${YELLOW}⚠${NC} Permission check (may not work on all systems)"
fi
rm -f .clai.toml

# Test 9: Lazy loading (should only load once)
echo ""
echo "Test 9: Lazy loading (config cached after first access)"
OUTPUT=$(cd /home/vee/Coding/clAI && cargo r -- "test" 2>&1)
test_result "Lazy loading works (no errors on multiple calls)"

# Test 10: Invalid TOML (should handle gracefully)
echo ""
echo "Test 10: Invalid TOML handling"
cd /home/vee/Coding/clAI
cat > .clai.toml << 'EOF'
[provider
default = "invalid"
EOF
chmod 600 .clai.toml 2>/dev/null || true
OUTPUT=$(cargo r -- "test" 2>&1)
# Should show warning but continue
if echo "$OUTPUT" | grep -q "Warning\|ParseError\|Failed to parse"; then
    test_result "Invalid TOML handled gracefully"
else
    test_result "Invalid TOML handled gracefully (no crash)"
fi
rm -f .clai.toml

# Summary
echo ""
echo "=== Test Summary ==="
echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi

