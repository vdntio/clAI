# Configuration System Test Results

## Manual Test Commands

### 1. Default Configuration (No Files)
```bash
cargo r -- "test"
```
**Expected:** Loads with default values (provider: "openrouter", max_files: 10, etc.)

### 2. CLI Flag Overrides
```bash
# Provider override
cargo r -- --provider "test-provider" "test"

# Model override
cargo r -- --model "gpt-4" "test"

# Combined
cargo r -- --provider "openai" --model "gpt-4" "test"
```
**Expected:** CLI flags take highest priority

### 3. Environment Variable Overrides
```bash
# Provider
CLAI_PROVIDER_DEFAULT="env-provider" cargo r -- "test"

# Context settings
CLAI_CONTEXT_MAX_FILES="30" CLAI_CONTEXT_MAX_HISTORY="5" cargo r -- "test"

# UI settings
CLAI_UI_COLOR="never" cargo r -- "test"
```
**Expected:** Environment variables override file configs but not CLI flags

### 4. Config File (Current Directory)
```bash
# Create config file
cat > .clai.toml << 'EOF'
[provider]
default = "file-provider"

[context]
max-files = 25
max-history = 5
EOF
chmod 600 .clai.toml

# Test
cargo r -- "test"

# Cleanup
rm -f .clai.toml
```
**Expected:** Config file loads and overrides defaults

### 5. XDG Config Directory
```bash
# Create XDG config
mkdir -p ~/.config/clai
cat > ~/.config/clai/config.toml << 'EOF'
[provider]
default = "xdg-provider"
EOF
chmod 600 ~/.config/clai/config.toml

# Test
cargo r -- "test"

# Cleanup
rm -f ~/.config/clai/config.toml
```
**Expected:** XDG config loads (lower priority than ./.clai.toml)

### 6. Precedence Test (CLI > Env > File > Default)
```bash
# Create file config
cat > .clai.toml << 'EOF'
[provider]
default = "file-provider"
EOF
chmod 600 .clai.toml

# Test precedence
CLAI_PROVIDER_DEFAULT="env-provider" cargo r -- --provider "cli-provider" "test"

# Cleanup
rm -f .clai.toml
```
**Expected:** CLI provider "cli-provider" wins (highest priority)

### 7. Permission Check
```bash
# Create file with insecure permissions
cat > .clai.toml << 'EOF'
[provider]
default = "test"
EOF
chmod 644 .clai.toml

# Test (should show warning)
cargo r -- "test"

# Cleanup
rm -f .clai.toml
```
**Expected:** Warning about insecure permissions (0600 required on Unix)

### 8. Invalid TOML Handling
```bash
# Create invalid TOML
cat > .clai.toml << 'EOF'
[provider
default = "invalid"
EOF
chmod 600 .clai.toml

# Test (should handle gracefully)
cargo r -- "test"

# Cleanup
rm -f .clai.toml
```
**Expected:** Warning about parse error, but continues with defaults

### 9. Lazy Loading
```bash
# First call (loads config)
cargo r -- "test"

# Second call (uses cache)
cargo r -- "test"
```
**Expected:** Both calls work, config is cached after first access

### 10. Multiple Config Files (Precedence)
```bash
# Create local config
cat > .clai.toml << 'EOF'
[context]
max-files = 20
EOF
chmod 600 .clai.toml

# Create XDG config
mkdir -p ~/.config/clai
cat > ~/.config/clai/config.toml << 'EOF'
[context]
max-files = 15
EOF
chmod 600 ~/.config/clai/config.toml

# Test (local should override XDG)
cargo r -- "test"

# Cleanup
rm -f .clai.toml ~/.config/clai/config.toml
```
**Expected:** Local config (./.clai.toml) overrides XDG config

## Expected Behavior Summary

1. **Precedence Order:**
   - CLI flags (highest)
   - Environment variables (CLAI_*)
   - Config files (./.clai.toml > $XDG_CONFIG_HOME/clai/config.toml > ~/.config/clai/config.toml > /etc/clai/config.toml)
   - Defaults (lowest)

2. **Security:**
   - Config files must have 0600 permissions on Unix
   - Insecure permissions generate warnings but don't stop execution

3. **Error Handling:**
   - Config loading errors go to stderr
   - Invalid TOML generates warnings but continues with defaults
   - Missing config files fall back to defaults

4. **Performance:**
   - Config is lazy-loaded (only on first access)
   - Config is cached after first load
   - Subsequent calls use cached config

