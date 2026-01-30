# clai — Product Requirements Document & Technical Specification for TypeScript Port

This document is a comprehensive PRD and technical specification derived from the Rust codebase. It is intended to support a full port of **clai** (AI-powered shell command translator) to TypeScript/Node.js. Every module, data flow, edge case, and behavioral detail is documented so the TypeScript implementation can match behavior exactly.

---

## Table of Contents

1. [Product Overview](#1-product-overview)
2. [Application Flow (End-to-End)](#2-application-flow-end-to-end)
3. [CLI (Argument Parsing)](#3-cli-argument-parsing)
4. [Configuration System](#4-configuration-system)
5. [Context Gathering](#5-context-gathering)
6. [AI Layer: Prompts, Requests, and Providers](#6-ai-layer-prompts-requests-and-providers)
7. [OpenRouter Integration](#7-openrouter-integration)
8. [Safety: Dangerous Commands](#8-safety-dangerous-commands)
9. [Output and Execution](#9-output-and-execution)
10. [Error Handling and Exit Codes](#10-error-handling-and-exit-codes)
11. [Signals, TTY, and Environment](#11-signals-tty-and-environment)
12. [Logging and Debug](#12-logging-and-debug)
13. [Color and Locale](#13-color-and-locale)
14. [Edge Cases and Behavioral Summary](#14-edge-cases-and-behavioral-summary)
15. [Data Structures Reference](#15-data-structures-reference)

---

## 1. Product Overview

### 1.1 Purpose

**clai** is a CLI that converts natural language instructions into executable shell commands using an AI provider (currently OpenRouter). It follows Unix philosophy: simple, composable, and privacy-respecting.

- **Input:** A single required positional argument: the natural language instruction (e.g. `"find all rust files modified today"`).
- **Output:** Either a single command string or multiple options; output is written to **stdout only** so it can be piped (e.g. `clai "..." | bash`).
- **Side channels:** All logs, warnings, prompts, and debug info go to **stderr**. Stdout is reserved for the generated command(s).

### 1.2 Design Principles (from codebase)

- **Pure functions preferred:** Most logic is pure; I/O is isolated to entry points and specific handlers.
- **Immutable config:** Runtime config is built once from CLI + file config and not mutated.
- **Strict stdout/stderr separation:** stdout = commands only; stderr = logs, warnings, prompts, debug.
- **Fail-safe safety:** If dangerous-command regex compilation fails, treat the command as dangerous (do not allow by default).
- **Lazy loading:** File config and provider instances are loaded/cached on first use.

---

## 2. Application Flow (End-to-End)

### 2.1 Main Entry Sequence

1. **Signal setup**  
   Register handlers for SIGINT, SIGTERM (and rely on default SIGPIPE behavior). Store an “interrupted” flag (e.g. `Arc<AtomicBool>`) that gets set when these signals fire.

2. **Early interruption check**  
   If interrupted before starting, exit with code **130** (Interrupted).

3. **Parse CLI**  
   Parse argv into a `Cli` struct. On parse failure, convert to `ClaiError::Usage` (exit 2) or `ClaiError::HelpOrVersion` (print to stdout, exit 0).

4. **Offline mode**  
   If `--offline` is set, immediately return a **fatal error** (e.g. “Offline mode is not yet supported”) and exit (e.g. code 1). No AI call is made.

5. **Load file config**  
   Call `get_file_config(&cli)`:
   - **Missing config file(s):** Non-fatal; use `FileConfig` defaults and optionally log “No config file found, using defaults” when verbose ≥ 1.
   - **Parse/permission errors:** Fatal; `ClaiError::Config` → exit 3.

6. **Build runtime config**  
   Build `Config` from `Config::from_cli(cli)`. Then:
   - If file config has `ui.interactive == true` and CLI did **not** set `--interactive`, set `config.interactive = true` (file can enable interactive; CLI can only add, not remove).
   - Resolve debug log path: CLI `--debug-file` overrides; else use file config `ui.debug_log_file` (expand `~`); else no file logging.

7. **Initialize file logger (optional)**  
   If a debug log path is set, create `FileLogger` and register it globally for the OpenRouter provider. On failure, **warn to stderr** and continue (non-fatal).

8. **Handle CLI logic**  
   Call `handle_cli(config, file_config, interrupt_flag)` (see below). Map errors to the appropriate `ClaiError` and exit with the corresponding code.

9. **Post-run**  
   If interrupted after `handle_cli`, exit 130. On success, exit 0. On error, log error to file (if logger exists), then print error (and optional backtrace if verbose ≥ 1) to stderr and exit with `ClaiError::exit_code()`.

### 2.2 handle_cli (Core Business Logic)

- **Interruption:** If interrupted at the start, return error (e.g. “Interrupted by signal”) → exit 130.
- **Logger:** Build a `Logger` from `Config` (verbosity + color).
- **Debug/config logging:** If verbose ≥ 2, log “Parsed config: …” at debug level; if verbose == 1, at info level. All to stderr.
- **Interruption:** Check again before AI call.
- **Generate commands:**
  - If `num_options > 1` **and** `interactive`: call `generate_commands(config)` → `Vec<String>`.
  - Else: call `generate_command(config)` → single `String`, then wrap in `vec![cmd]`.
- **AI errors:** Map to `ClaiError::API`; if the error message contains a parenthesized HTTP status e.g. `(401)` or `(429)`, parse it and attach as `status_code` on the error.
- **Interruption:** Check again before output.
- **Dry-run:** If `--dry-run`:
  - For each command in the list, call `print_command(cmd)` (with newline between commands, and a final newline after the last).
  - Then return success (no safety prompts, no execution).
- **Dangerous check:** Compute `is_dangerous = is_dangerous_command(&first_command, &file_config)`.
- **Interactive TTY:** `is_interactive_mode = config.interactive && is_interactive()` (both stdin and stdout are TTY).
- **Branch:**
  - **If dangerous:**
    - Compute `should_prompt_user = should_prompt(&cli, &file_config)` (TTY, `confirm_dangerous`, and not `--force`).
    - If **should prompt:** run `handle_dangerous_confirmation(&first_command, &config)`:
      - **Execute** → `print_command(&first_command)` then Ok.
      - **Copy** → same (print command) then Ok.
      - **Abort** → return `ClaiError::Safety("Command rejected by user")` → exit 5.
      - **Error (e.g. EOF)** → return `ClaiError::Safety("Error during confirmation: … Command rejected.")` → exit 5.
    - If **not** prompting (piped / force / confirm disabled): print first command and Ok.
  - **Else if interactive mode (and not dangerous):**
    - Call `prompt_command_action(&commands, &config)`:
      - **(Execute, cmd)** → run `execute_command(&cmd)`; if exit code ≠ 0, return general error with “Command exited with code N”.
      - **(Output, cmd)** → `print_command(&cmd)` then Ok.
      - **(Abort, _)** → return `ClaiError::Safety("Command rejected by user")` → exit 5.
      - **Error (e.g. not TTY):** warn to stderr, then print first command and Ok (fallback to output).
  - **Else (safe, non-interactive):**
    - Print first command via `print_command(&first_command)` and Ok.

All `print_command` and `execute_command` I/O errors are wrapped in a general error (e.g. “Failed to write command to stdout” / “Failed to execute command”).

---

## 3. CLI (Argument Parsing)

### 3.1 Arguments (clap-derived)

| Argument | Type | Default | Description |
|----------|------|---------|-------------|
| `instruction` | positional, required | — | Natural language instruction. |
| `-m, --model` | optional string | — | Override AI model. |
| `-p, --provider` | optional string | — | Override AI provider. |
| `-q, --quiet` | flag | false | Suppress non-essential output. |
| `-v, --verbose` | count | 0 | Increase verbosity (multiple times). |
| `--no-color` | flag | false | Disable color (deprecated in favor of `--color=never`). |
| `--color` | enum | `auto` | `auto` \| `always` \| `never`. |
| `-i, --interactive` | flag | false | Prompt for execute/copy/abort on dangerous; in safe interactive mode, prompt for command choice. |
| `-f, --force` | flag | false | Skip dangerous-command confirmation. |
| `-n, --dry-run` | flag | false | Show command(s) only; no safety prompt, no execution. |
| `-c, --context` | optional string | — | **Additional context file path** — stored in config but **not** currently read/injected into the prompt in the Rust code; document for port. |
| `--offline` | flag | false | If set, app exits with “not yet supported” before any AI call. |
| `-o, --options` | number | 3 | Number of command options (1–10). |
| `-d, --debug` | flag | false | Print to stderr the request (model, messages, etc.) that would be sent to the AI. |
| `--debug-file` | optional string | — | If present: empty string means use default path; non-empty means path (supports `~`). Enables file logging. |

### 3.2 Config Derived from CLI

- `num_options` is **clamped** to 1–10.
- If `--no-color` is set, effective color is **Never** regardless of `--color`.
- `debug_log_file`: `None` if `--debug-file` not given; if given, resolve path (empty → default cache path, e.g. `~/.cache/clai/debug.log`).

---

## 4. Configuration System

### 4.1 File Config (TOML) Structure

- **Discovery order (highest priority first):**
  1. `./.clai.toml`
  2. `$XDG_CONFIG_HOME/clai/config.toml` (if set and non-empty)
  3. `~/.config/clai/config.toml` (e.g. via `directories` / home dir)
  4. `/etc/clai/config.toml`

Only **existing** paths are loaded. Merging order in the **loader** is from **lowest** to **highest** priority (so highest priority wins when using the loader’s simple merge). The **merger** then applies: defaults → file config → env → CLI.

### 4.2 File Config Schema (kebab-case in TOML)

- **`[provider]`**
  - `default`: string, default `"openrouter"`.
  - `fallback`: array of provider names (default `[]`).
- **`[context]`**
  - `max-files`: number, default 10.
  - `max-history`: number, default 3.
  - `redact-paths`: bool, default false.
  - `redact-username`: bool, default false.
- **`[safety]`**
  - `dangerous-patterns`: array of strings (regex). If empty, code uses **built-in** patterns (see Safety).
  - `confirm-dangerous`: bool, default true.
- **`[ui]`**
  - `color`: string, default `"auto"`.
  - `debug-log-file`: optional string.
  - `interactive`: bool, default false.
- **Provider-specific tables** (e.g. `[openrouter]`), flattened:
  - `api-key`: optional string (in-file key; rely on 0600 permissions).
  - `api-key-env`: optional string (env var name).
  - `model`: optional string.
  - `endpoint`: optional string (for local providers).

### 4.3 Loading and Security

- **Existence:** If no config file exists, use defaults (non-fatal).
- **Permissions (Unix):** Config file must be **0600**. Otherwise `ConfigLoadError::InsecurePermissions` → fatal (exit 3).
- **Read/parse errors:** Fatal → exit 3.
- **Env var references in API keys:** Support `${VAR_NAME}` and `$VAR_NAME` in config values; resolve via `std::env::var`. Used when resolving provider API key from file.

### 4.4 Merge Precedence (Final Merged FileConfig)

1. **Defaults** (lowest).
2. **Config files** (loader merges existing files; merger then uses that as one layer).
3. **Environment:** `CLAI_*` vars; keys lowercased, e.g. `CLAI_PROVIDER_DEFAULT` → `provider_default`. Sections/fields mapped (e.g. `context_max_files`, `safety_confirm_dangerous`). Booleans: parse; on failure use sensible default (e.g. true for confirm).
4. **CLI** (highest): e.g. `--provider`, `--model` override merged file config. Model is written into the corresponding provider’s config (e.g. `providers["openrouter"].model`).

### 4.5 Caching

- **File config:** Loaded once per process and cached (e.g. lazy static + Mutex). Subsequent `get_file_config(cli)` return the cached result (CLi is used for the merge at first load only; cache key is effectively “first call”).

---

## 5. Context Gathering

### 5.1 Purpose

Produce a single JSON string containing system info, cwd, file list, shell history, and optional stdin. This is then parsed in the AI handler and turned into the **prompt** text.

### 5.2 Steps (gather_context)

1. **System info**  
   Cached per run. Fields: `os_name`, `os_version`, `architecture`, `shell`, `user`, `total_memory`.  
   - Shell from `$SHELL` (basename).  
   - User from `$USER` or `$USERNAME`.  
   - Architecture from runtime (e.g. `process.arch`).

2. **CWD**  
   Current working directory; if `redact_paths` is true, apply path redaction (see below).

3. **Files**  
   `scan_directory(max_files, redact_paths)` from CWD: read directory, sort by name, take first `max_files`, truncate each path to 80 chars (by basename if longer), optionally redact.

4. **History**  
   `get_shell_history(max_history)`: detect shell from `$SHELL`, map to history file path:
   - **bash:** `~/.bash_history`
   - **zsh:** `~/.zsh_history`
   - **fish:** `~/.local/share/fish/fish_history`
   - Other shells: no history file (return empty).  
   Read **last N lines** using a tail-like read (e.g. seek from end minus 4096 bytes, then take last N lines). The Rust code does **not** parse fish’s `- cmd: ...` format; it just reads raw lines. For the port, matching this behavior is sufficient; parsing fish format can be an optional enhancement.

5. **Stdin**  
   Only if stdin is **not** a TTY (i.e. piped). Read up to 10 KiB; UTF-8 with lossy replacement for invalid bytes. If not piped or error, use `None`.

6. **Redaction (when enabled)**  
   - Replace `$HOME`, `~/`, `~`, and `/home/<user>/` with `[REDACTED]` (or equivalent) in paths and in CWD.

### 5.3 Output JSON Shape

- `system`: object (string key/values).
- `cwd`: string.
- `files`: array of strings.
- `history`: array of strings.
- `stdin`: string or null.

Pretty-printed (e.g. 2-space indent) for readability in debug; can be minimized for production if desired.

### 5.4 Optional Context File (--context)

CLI has `-c/--context` pointing to a file path. This is stored in `Config` and passed through but **not** read or appended to the prompt in the current Rust code. For the port, you may choose to read this file and append its contents to the user instruction or to a dedicated “Additional context” section in the prompt.

---

## 6. AI Layer: Prompts, Requests, and Providers

### 6.1 Prompt Construction (build_prompt)

- **System context:** JSON string (from context’s `system`).
- **Directory context:** One line “Current directory: …” and “Files: …” (e.g. count or list).
- **Recent Shell History:** Numbered list of history lines (if non-empty).
- **User instruction:** “User Instruction: ” + instruction.
- **System instruction (fixed):** “Respond ONLY with the executable command. Do not include markdown code fences, explanations, or any other text. Just the command itself.”
- If **stdin** was provided, append “\n\nStdin input: ” + content.

### 6.2 Single-Command Request (build_chat_request)

- **Messages:**
  - System: “You are a helpful assistant that converts natural language instructions into executable shell commands. Respond with ONLY the command, no explanations or markdown.”
  - User: the full prompt string from above.
- **Model:** From config (CLI or file); optional on the request.
- **Temperature / max_tokens:** Left to provider defaults if not set.

### 6.3 Multi-Command Request (build_multi_chat_request)

- **System message:** Instructs to generate **exactly N** different command options and to respond **only** with a JSON object: `{"commands": ["cmd1", "cmd2", ...]}`. Rules: exactly N commands, no markdown/explanations, order from simplest to more advanced.
- **User message:** Same full prompt as single-command.
- **Model / temperature / max_tokens:** Same as above.

### 6.4 Response Parsing

- **Single command:**  
  - Regex to extract content from markdown code fences: ` ```(?:bash|sh|shell)?\s*\n(.*?)\n?``` `.  
  - If match: use trimmed capture as the command.  
  - Else: use full response trimmed.

- **Multiple commands:**  
  - Trim response.  
  - If wrapped in ` ``` `, strip ` ```json ` or ` ``` ` and parse inner JSON.  
  - Parse as `{ commands: string[] }`.  
  - If that fails, try parsing as a bare array `string[]`.  
  - If that fails, try to find `{ ... }` in the text and parse that.  
  - If all fail, fallback to **single-command** extraction and return `[single_cmd]`.  
  - Empty `commands` or all-whitespace entries filtered out; if none left, error (“AI returned no commands” or “empty commands array”).

### 6.5 Provider Chain

- **Order:** `[default_provider] + fallback` (default first, no duplicate).
- **Model string:** If the user/model string contains `provider/model` (e.g. `openrouter/gpt-4o`), use that provider and model; else use default provider and the string as model.
- **Lazy init:** Each provider is created on first use (e.g. when that slot is tried in the chain).
- **Completion:** Try each provider in order; on success return; on failure try next. If all fail, return last error.

---

## 7. OpenRouter Integration

### 7.1 Endpoint and Format

- **URL:** `https://openrouter.ai/api/v1/chat/completions`
- **Format:** OpenAI-compatible request/response (see types below).

### 7.2 Authentication and Config

- **API key resolution order:**  
  1) `providers["openrouter"].api_key` (from config),  
  2) `providers["openrouter"].api_key_env` (env var name from config),  
  3) `OPENROUTER_API_KEY` env var.  
- **Model:** From request; else provider default from config; else `"qwen/qwen3-coder"`.

### 7.3 Request

- **Headers:**  
  `Authorization: Bearer <key>`, `Content-Type: application/json`, optional `HTTP-Referer`, `X-Title`.
- **Body:**  
  `model`, `messages` (array of `{ role, content }`), optional `temperature`, optional `max_tokens`.

### 7.4 Response Handling

- **Success:** Parse JSON; take first choice’s `message.content`; map `usage` if present.
- **HTTP errors:**  
  - 401/403: “Authentication error (N): …”  
  - 429: “Rate limit error (N): …”  
  - 408/504: “Timeout error (N): …”  
  - Other: “API error (N): …”  
  Include response body in message. Status code is also captured for `ClaiError::API` (e.g. for exit or logging).

### 7.5 Retries

- On **429** only: retry up to 3 times with exponential backoff (e.g. 1s, 2s, 4s). Other errors are not retried.

### 7.6 File Logging (when enabled)

- Before send: log request (model, messages, temperature, max_tokens).
- After response: log response (model, status, content, usage).
- On network/parse/API errors: log error with context (e.g. status_code, url).

---

## 8. Safety: Dangerous Commands

### 8.1 Detection

- **Source of patterns:** File config `safety.dangerous_patterns`. If **empty**, use **built-in** list (see below).
- **Compilation:** Each pattern is a **regex**. If **any** pattern fails to compile, the whole compilation fails; the detector then treats the command as **dangerous** (fail-safe).
- **Matching:** Command is dangerous if **any** compiled regex matches the command string (substring match).

### 8.2 Default Patterns (regex)

- `rm\s+-rf\s+/`
- `rm\s+-rf\s+/\s*$`
- `dd\s+if=/dev/zero`
- `mkfs\.\w+\s+/dev/`
- `sudo\s+rm\s+-rf\s+/`
- `>\s*/dev/`
- `format\s+[c-z]:`
- `del\s+/f\s+/s\s+[c-z]:\\`

(Exact list from `safety/patterns.rs`.)

### 8.3 When to Prompt (Dangerous)

- **should_prompt:** true only if:  
  - stdin is TTY **and** stdout is TTY, **and**  
  - `safety.confirm_dangerous` is true, **and**  
  - `--force` is false.

### 8.4 Dangerous Confirmation Prompt

- Printed on **stderr**.  
- Warning line: e.g. “⚠️  DANGEROUS: <command>” (optionally colored).  
- Prompt: “\[E]xecute/[C]opy/[A]bort? ”  
- Read one line from stdin; trim; first character case-insensitive:  
  - E → Execute  
  - C → Copy  
  - A → Abort  
- Empty input or EOF → treat as **Abort**.  
- Invalid character → error (e.g. “Invalid input: … Expected E, C, or A”).

### 8.5 Interactive Safe-Command Prompt (prompt_command_action)

- Used when **not** dangerous and **interactive** (and multiple commands or single with execute/output choice).
- **Requires stderr TTY.** If not TTY, return “Output” with first command (no prompt).
- Display: one line per option, e.g. “❯ <command> [1/3] (Tab: next | Enter | ^C)”.  
- **Tab:** cycle to next command (replace line in place; account for line wrapping when clearing).  
- **Enter:** Execute selected command.  
- **Ctrl+C / Esc:** Abort.  
- Line count for clearing is computed from terminal width and text width (wrap-aware).

### 8.6 Command Execution (execute_command)

- **Unix:** `$SHELL -c "<command>"` (default shell `/bin/sh` if `SHELL` unset).  
- **Windows:** `cmd /C <command>`.  
- Return the process exit code (or 1 if not available).

---

## 9. Output and Execution

### 9.1 print_command(command)

- **If stdout is TTY:** print `command.trim()` + newline.  
- **If stdout is piped:** print `command.trim()` **without** trailing newline (so piping to `bash` or `xclip` works cleanly).  
- Always flush stdout.

### 9.2 Dry-Run

- When `--dry-run`, **all** generated commands are printed (one per “block”, with newlines between and after), and no safety or execution logic runs.

### 9.3 Single vs Multiple Commands

- **Single:** One command string; first (and only) element is used for dangerous check and output.  
- **Multiple:** Used when `num_options > 1` and `interactive`. First command is used for dangerous check; user can cycle and then Execute or Output one, or Abort.

---

## 10. Error Handling and Exit Codes

### 10.1 ClaiError Variants

- **General(anyhow):** exit 1.  
- **Usage(string):** exit 2 (e.g. invalid CLI).  
- **Config { source }:** exit 3.  
- **API { source, status_code? }:** exit 4.  
- **Safety(string):** exit 5.  
- **HelpOrVersion(string):** exit 0 (help/version printed to stdout).

### 10.2 Interrupted

- When the “interrupted” flag is set (SIGINT/SIGTERM), exit **130**.

### 10.3 Help / Version

- On `--help` or `--version`, print to **stdout** and exit 0. Do not treat as error.

### 10.4 Error Reporting

- On non–help/version error: if file logger is set, log the error (with variant and message/context). Then print error to **stderr**. If verbose ≥ 1, append backtrace (cause chain).

### 10.5 HTTP Status in API Errors

- If the API error message contains a parenthesized 3-digit code e.g. `(401)`, parse it and attach to `ClaiError::API` for logging/exit (e.g. for rate-limit handling).

---

## 11. Signals, TTY, and Environment

### 11.1 Signals

- **SIGINT / SIGTERM:** Set shared “interrupted” flag. Do not exit immediately inside the handler; check the flag at defined points (before run, after parse, before AI, before output, after handle_cli) and then exit 130.
- **SIGPIPE:** No custom handler; rely on default (avoid writing to broken pipe).

### 11.2 TTY Checks

- **is_stdin_tty()**, **is_stdout_tty()**, **is_stderr_tty():** Used for:  
  - Interactive mode: both stdin and stdout TTY.  
  - Dangerous prompt: need TTY for input.  
  - Safe interactive prompt: stderr TTY for display.  
  - Piped output: stdout not TTY → no trailing newline in `print_command`.  
  - Color: often stderr TTY for logs/prompts.

### 11.3 Environment Variables (Summary)

- **OPENROUTER_API_KEY:** Default env for OpenRouter.  
- **NO_COLOR:** Disable color (no-color.org).  
- **CLICOLOR:** 0 = no color, 1 = color.  
- **TERM=dumb:** No color.  
- **XDG_CONFIG_HOME:** Config path.  
- **HOME, USER, USERNAME, SHELL:** System and history paths.  
- **LANG:** Locale (for locale module).  
- **CLAI_*:** Config overrides (see Configuration).

---

## 12. Logging and Debug

### 12.1 Stderr Logger (Logger)

- Built from `Config`: quiet → Error level; else verbosity 0 → Warning, 1 → Info, 2 → Debug, 3+ → Trace.  
- Color: from color mode (auto/always/never).  
- All log methods write to **stderr** with level prefix (and optional color).

### 12.2 Debug Flag (--debug)

- When set, before sending the request to the AI, print to **stderr** a human-readable dump: model, temperature, max_tokens, and full messages (role + content). No file logging required for this.

### 12.3 File Logger (--debug-file / ui.debug_log_file)

- **Path:** Default `~/.cache/clai/debug.log` (or overridden).  
- **Format:** JSON Lines. Each line: `{ "ts": "<ISO8601>", "level": "DEBUG"|"ERROR", "event": "<event>", ... }`.  
- **Events:** e.g. `ai_request`, `ai_response`, `network_error`, `api_error`, `parse_error`, `general_error`, etc., with event-specific payloads.  
- **Size:** Truncate/recreate file if it exceeds 10 MB.  
- **Thread-safety:** Append under a lock.  
- **Creation:** Parent directories created if needed.

---

## 13. Color and Locale

### 13.1 Color

- **Modes:** Auto, Always, Never.  
- **--no-color** forces Never.  
- **Auto:** If `CLICOLOR=0` → no; if `NO_COLOR` set → no; if `TERM=dumb` → no; else use stderr TTY.  
- Used for: stderr logs, dangerous warning, interactive prompt (❯, command, hints).

### 13.2 Locale

- **get_locale():** From `LANG`, default `"en_US"`.  
- **get_language_code():** First segment before `_` or `.`.  
- **is_c_locale():** true if `LANG` is `C` or `POSIX`.  
- (Used for potential i18n; not heavily used in the Rust code path.)

---

## 14. Edge Cases and Behavioral Summary

### 14.1 Config

- No config file → use defaults; optional “No config file found” at verbose ≥ 1.  
- Config file not 0600 (Unix) → fatal.  
- Invalid TOML or read error → fatal.  
- Env `CLAI_*` overrides file; CLI overrides env.  
- Empty `dangerous_patterns` → use built-in regex list.

### 14.2 Context

- CWD unreadable → error in gather_context (propagates).  
- Directory scan failure → return empty list (no crash).  
- History file missing or unreadable → empty history.  
- Stdin only read when piped; max 10 KiB; invalid UTF-8 replaced with replacement char.  
- Redaction: `~/`, `$HOME`, `/home/<user>/` → `[REDACTED]`.

### 14.3 AI

- Model “provider/model” → use that provider; else default provider.  
- Provider chain: try in order; return first success or last failure.  
- Single command: strip markdown fences; fallback full trim.  
- Multi-command: try JSON `{ commands }`, then bare array, then extract `{ }` from text, then single-command fallback. Empty or all-whitespace commands → error.  
- OpenRouter 429 → retry with backoff (up to 3 times).  
- Timeout: 60 s (configurable in code).

### 14.4 Safety

- Regex compile failure → treat command as dangerous.  
- Dangerous + TTY + confirm_dangerous + not force → prompt (E/C/A).  
- Dangerous + piped/force/confirm off → print first command (no prompt).  
- Safe + interactive + TTY → prompt (Tab/Enter/Esc).  
- Safe + interactive but not TTY → print first command (with warning).  
- Safe + non-interactive → print first command.

### 14.5 Output

- Stdout: **only** the command(s).  
- Piped stdout → no trailing newline for the last command.  
- TTY stdout → trailing newline.  
- Dry-run: all commands, newlines between and after.

### 14.6 Exit and Signals

- Help/version → stdout, exit 0.  
- Interrupted → exit 130.  
- General 1, Usage 2, Config 3, API 4, Safety 5.  
- On error, log to file (if enabled), then stderr (and backtrace if verbose ≥ 1).

---

## 15. Data Structures Reference

### 15.1 CLI (Input)

- `instruction: string`  
- `model?: string`  
- `provider?: string`  
- `quiet: boolean`  
- `verbose: number` (count)  
- `no_color: boolean`  
- `color: "auto" | "always" | "never"`  
- `interactive: boolean`  
- `force: boolean`  
- `dry_run: boolean`  
- `context?: string`  
- `offline: boolean`  
- `num_options: number` (1–10)  
- `debug: boolean`  
- `debug_file?: string`

### 15.2 Config (Runtime, from CLI + file)

- Same as CLI where applicable, plus:  
- `num_options` clamped 1–10  
- `color` resolved (no_color → Never)  
- `debug_log_file?: PathBuf` (optional log path)

### 15.3 FileConfig (TOML + env + CLI merged)

- `provider: { default: string, fallback: string[] }`  
- `context: { max_files, max_history, redact_paths, redact_username }`  
- `safety: { dangerous_patterns: string[], confirm_dangerous: boolean }`  
- `ui: { color: string, debug_log_file?: string, interactive: boolean }`  
- `providers: Record<string, { api_key?, api_key_env?, model?, endpoint? }>`

### 15.4 ContextData (Internal)

- `system: Record<string, string>`  
- `cwd: string`  
- `files: string[]`  
- `history: string[]`  
- `stdin?: string`

### 15.5 Chat Types

- **Role:** `system` | `user` | `assistant`  
- **ChatMessage:** `{ role, content }`  
- **ChatRequest:** `{ messages, model?, temperature?, max_tokens? }`  
- **ChatResponse:** `{ content, model?, usage? }`  
- **Usage:** `{ prompt_tokens, completion_tokens, total_tokens }`

### 15.6 OpenRouter (OpenAI-Compatible)

- **Request:** `{ model, messages: { role, content }[], temperature?, max_tokens? }`  
- **Response:** `{ model, choices: { index, message: { role, content }, finish_reason? }[], usage? }`  
- **Usage:** `{ prompt_tokens, completion_tokens, total_tokens }`

### 15.7 Safety

- **Decision:** Execute | Copy | Abort  
- **CommandAction:** Execute | Output | Abort  
- **ConfirmationError:** Eof | InvalidInput(string) | IoError(string)  
- **InteractiveError:** Eof | IoError(string) | NotTty | NoCommands  

---

## 16. What Gets Sent Where (Quick Reference)

| Data | Destination | When |
|------|-------------|------|
| Generated command(s) | **stdout** | Always (unless error/abort). Piped = no trailing newline; TTY = newline. |
| Help / version | **stdout** | `--help` / `--version` only. |
| Logs, warnings, debug dump, prompts (E/C/A, Tab/Enter) | **stderr** | Verbosity, debug, interactive, errors. |
| API request body | **OpenRouter** `POST /api/v1/chat/completions` | When generating command(s). |
| Request/response and errors | **Debug log file** (if enabled) | JSON Lines; path from `--debug-file` or config. |
| User confirmation input (E/C/A) | **stdin** | Only when dangerous and `should_prompt` (TTY, confirm_dangerous, not force). |
| Tab/Enter/Esc in interactive | **stderr TTY** (read_key) | Only when safe + interactive + multiple or execute choice. |

### 16.1 Dangerous Patterns: File vs Code Defaults

- **From config file:** `safety.dangerous_patterns` is an array of **strings** that are compiled as **regex**. If the user omits this key, TOML defaults (in `file.rs`) are used: e.g. `"rm -rf"`, `"sudo rm"`, `"mkfs"`, `"dd if="`, `"> /dev/"`, `"format"` — these are substring-style and may need escaping for regex in some engines.
- **When list is explicitly empty:** The **code** fallback in `patterns.rs` uses a stricter **regex** list (e.g. `rm\s+-rf\s+/`, `dd\s+if=/dev/zero`, etc.). So “no patterns” in config → use code’s built-in regex list; “patterns from file” → use those strings as regex (invalid regex → fail-safe: treat as dangerous).

---

## 17. Testing Hints for the TypeScript Port

- **Exit codes:** Run the Rust binary with invalid args, missing config, bad key, API failure, safety abort, and Ctrl+C; assert exact exit codes (0, 1, 2, 3, 4, 5, 130).
- **Stdout purity:** Pipe Rust stdout to a file; ensure only the command line(s) and optional trailing newline (when TTY) appear. No logs or prompts.
- **Piped vs TTY:** Compare Rust output when stdout is TTY vs piped (e.g. `clai "ls" > out` vs `clai "ls" | cat`); trailing newline behavior should match.
- **Dry-run:** Ensure `--dry-run` prints all options (when multiple) with no confirmation and no execution.
- **Dangerous:** With a TTY, force a dangerous command (e.g. “rm -rf /”); ensure E/C/A prompt appears and that Abort returns exit 5. With `--force` or piped, ensure no prompt and command is printed.
- **Multi-command + interactive:** Request multiple options with `-o 3 -i` and verify Tab cycles and Enter executes one command.
- **Config precedence:** Set env `CLAI_PROVIDER_DEFAULT`, then override with `--provider`; CLI should win. Same for model and safety/context settings.
- **Regex safety:** Use a config with an invalid regex in `dangerous_patterns`; the app should treat commands as dangerous (or fail config load, depending on where you validate).

---

This document and the codebase are the single source of truth for the Rust implementation. For the TypeScript port, replicate these flows, data structures, and edge behaviors so that CLI output and exit codes remain consistent across environments.
