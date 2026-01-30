# clai — Product Requirements Document & Technical Specification

**Purpose:** Complete specification for porting the Rust `clai` CLI to TypeScript/Node. Covers product intent, architecture, data flow, config, OpenRouter, context, safety, and every edge case implemented in the Rust codebase.

---

## Table of Contents

1. [PRD](#part-1-product-requirements-document-prd) — Overview, goals, user stories
2. [Architecture](#part-2-architecture-overview) — Flow, module map
3. [What Gets Sent Where](#part-3-what-gets-sent-where) — AI request/response, stdout/stderr, debug log
4. [Configuration](#part-4-configuration-full-specification) — Precedence, paths, schema, env, security
5. [OpenRouter Integration](#part-5-openrouter-integration-technical) — API, retries, model selection, chain
6. [Context Gathering](#part-6-context-gathering) — System, directory, history, stdin, redaction
7. [Safety](#part-7-safety-dangerous-commands) — Patterns, prompt conditions, interactive flow
8. [CLI Reference](#part-8-cli-reference) — All flags and options
9. [Error Handling](#part-9-error-handling-and-exit-codes) — Exit codes, reporting, signals
10. [Output and Logging](#part-10-output-and-logging) — Print command, logger, file logger
11. [Signals and TTY](#part-11-signals-and-tty) — TTY/pipe, SIGINT/SIGTERM/SIGPIPE
12. [Edge Cases Checklist](#part-12-edge-cases-and-behaviors-checklist-for-port) — Config, context, AI, safety, output
13. [TypeScript Port Notes](#part-13-typescript-port-notes) — Layout, parity, optional enhancements

---

## Part 1: Product Requirements Document (PRD)

### 1.1 Product Overview

**Name:** clai (AI-powered shell command translator)

**One-liner:** A CLI that converts natural language into executable shell commands using an AI provider (OpenRouter).

**Example:**
```bash
$ clai "find all rust files modified today"
find . -name "*.rs" -mtime 0
```

**Philosophy:**
- Unix-native: composable, pipe-friendly, minimal.
- Privacy-respecting: configurable context (redaction, limits).
- Safety-first: dangerous-command detection and optional confirmation.

### 1.2 Goals

1. **Correctness:** Generated commands must be single, executable shell commands (or a user-chosen option when multiple are requested).
2. **Composability:** stdout = command only (suitable for piping); stderr = logs, warnings, prompts.
3. **Configurability:** Layered config (file + env + CLI), provider/model selection, context limits, safety patterns.
4. **Safety:** Detect dangerous patterns (e.g. `rm -rf /`, `dd if=`, etc.) and optionally prompt before output/execute.
5. **Observability:** Optional debug prompt (`-d`), optional file logging (`--debug-file`), verbosity levels.

### 1.3 User Stories

| ID | Story | Acceptance |
|----|--------|------------|
| US1 | As a user I run `clai "instruction"` and get one command on stdout. | Single line to stdout; no extra text. |
| US2 | As a user I run `clai -i "instruction"` and can execute, output, or abort. | Interactive prompt (TTY); Tab to cycle options when multiple. |
| US3 | As a user I run `clai -o 3 "instruction"` and get 3 options to choose from (when interactive). | Multiple commands; Tab/Enter/Esc behavior. |
| US4 | As a user I run `clai -n "instruction"` and only see the command(s), no execution. | Dry-run: print only; no safety prompt, no execute. |
| US5 | As a user I pipe: `clai "instruction" \| sh`. | stdout = command only, no trailing newline when piped. |
| US6 | As a user I configure provider/model/context/safety in a file. | File config applied; precedence: CLI > env > file. |
| US7 | As a user I get a warning when a dangerous command is generated. | Prompt [E]xecute/[C]opy/[A]bort when TTY + config allows. |
| US8 | As a user I use `--force` to skip dangerous confirmation. | Command is printed without prompt. |

### 1.4 Non-Goals (Current Rust Scope)

- Offline/local-only mode is **not** implemented (flag exists but returns an error).
- Clipboard copy is “copy” in the sense of “output to stdout” only (no system clipboard).
- No built-in shell execution in non-interactive mode (user pipes to `sh` if desired).

---

## Part 2: Architecture Overview

### 2.1 High-Level Flow

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌─────────────┐
│  CLI parse  │────▶│ Config build │────▶│  Context    │────▶│  Prompt     │
│  (clap)     │     │ (file+env+   │     │  gathering  │     │  build      │
│             │     │  CLI merge)  │     │             │     │             │
└─────────────┘     └──────────────┘     └─────────────┘     └──────┬──────┘
                                                                     │
┌─────────────┐     ┌──────────────┐     ┌─────────────┐            │
│  Output /   │◀────│  Safety       │◀────│  AI         │◀────────────┘
│  Execute    │     │  (dangerous?) │     │  (OpenRouter│
│             │     │  prompt?)     │     │   chain)    │
└─────────────┘     └──────────────┘     └─────────────┘
```

1. **Parse CLI** → `Cli` struct (instruction, flags).
2. **Build config** → Runtime `Config` from CLI; file config via `get_file_config(cli)` (cached, merged with env + CLI).
3. **Gather context** → System, cwd, files, history, stdin (respecting file config limits and redaction).
4. **Build prompt** → Single string: system + directory + history + instruction + “respond only with command”.
5. **AI** → Provider chain (e.g. OpenRouter); single or multi-command request.
6. **Safety** → If dangerous and conditions met, prompt [E]/[C]/[A]; else print or execute per mode.
7. **Output** → stdout = command(s); stderr = logs, warnings, prompts.

### 2.2 Module Map (Rust → Conceptual)

| Rust module      | Responsibility |
|------------------|----------------|
| `main`           | Signal setup, CLI parse, config load, `handle_cli`, exit codes. |
| `cli`            | Argument definitions and parsing (clap). |
| `config`         | FileConfig (TOML), Config (runtime), paths, loader, merger, cache. |
| `context`        | System info, directory scan, shell history, stdin. |
| `ai`             | Prompt build, chat types, provider trait, chain, OpenRouter. |
| `safety`         | Dangerous patterns (regex), detector, confirmation prompt, interactive (Tab/Enter/Esc), execute. |
| `error`          | ClaiError variants, exit codes, stderr print, file logging. |
| `output`         | Print command (with/without newline by TTY/pipe). |
| `logging`        | Logger from config, FileLogger (JSONL). |
| `signals`        | SIGINT/SIGTERM, TTY/pipe detection. |
| `color`          | Color mode (auto/always/never), NO_COLOR, TERM. |
| `locale`         | LANG, language code (currently minimal use). |

---

## Part 3: What Gets Sent Where

### 3.1 To the AI Provider (OpenRouter)

**Endpoint:** `POST https://openrouter.ai/api/v1/chat/completions`

**Headers:**
- `Authorization: Bearer <API_KEY>`
- `Content-Type: application/json`
- `HTTP-Referer: https://github.com/clai` (optional)
- `X-Title: clai` (optional)

**Body (OpenAI-compatible):**
- `model`: string (from CLI `--model`, or file config `[provider.openrouter] model`, or default `qwen/qwen3-coder`).
- `messages`: array of `{ role: "system"|"user"|"assistant", content: string }`.
- `temperature`: optional number (not set in current Rust).
- `max_tokens`: optional number (not set in current Rust).

**Message construction:**
- **Single command:**  
  - System: “You are a helpful assistant… Respond with ONLY the command, no explanations or markdown.”  
  - User: full prompt (system context + directory context + history + instruction + “Respond ONLY with the executable command…”).
- **Multiple commands (`-o N`):**  
  - System: instructs to return exactly N options as JSON: `{"commands": ["cmd1", "cmd2", …]}`.  
  - User: same prompt as above.

**Prompt contents (user message):**
- “System Context:” + JSON-like system info (OS, arch, shell, user, memory).
- “Directory Context:” + “Current directory: <cwd>” + “Files: <count or list>”.
- “Recent Shell History:” + numbered list (if any).
- “User Instruction: ” + user instruction.
- “Respond ONLY with the executable command…” (or equivalent for multi).

**What is NOT sent (when redaction is enabled in file config):**
- Full paths can be redacted (paths replaced with `[REDACTED]`); username can be redacted (config options: `context.redact_paths`, `context.redact_username`).

### 3.2 From the AI Provider

- Response: OpenAI-format with `choices[].message.content`.
- **Single command:** Content is parsed to strip markdown code fences (e.g. ```bash … ```); remainder (or full content if no fence) is the command.
- **Multiple commands:** Content is parsed as JSON `{"commands": ["…", "…"]}`; if wrapped in markdown, fences are stripped first; fallbacks: raw array `["…"]`, or single command extraction.

### 3.3 To the User (stdout vs stderr)

| Content              | Stream | When |
|----------------------|--------|------|
| Generated command(s)| stdout | Always (dry-run: all; non-dry-run: first or selected). |
| Trailing newline     | stdout | Only when stdout is a TTY; when piped, no trailing newline. |
| Debug prompt         | stderr | `-d` / `--debug`. |
| Verbosity / info     | stderr | `-v` / `-vv`. |
| “DANGEROUS” warning  | stderr | When dangerous and prompting. |
| [E]xecute/[C]opy/[A]bort | stderr | Same. |
| Interactive prompt (❯ cmd [1/N]) | stderr | Interactive mode. |
| Errors               | stderr | Always. |
| Help/version         | stdout | `--help` / `--version` (exit 0). |

### 3.4 To the Debug Log File (if enabled)

- **When:** `--debug-file` (CLI) or `ui.debug_log_file` (file config); path can be `~/...`.
- **Format:** JSON Lines (one JSON object per line).
- **Fields (conceptually):** `ts` (ISO8601), `level`, `event`, plus event-specific data.
- **Events:** `ai_request` (model, messages, temperature, max_tokens), `ai_response` (model, status, content, usage), `network_error`, `api_error`, `parse_error`, and general/usage/config/safety errors from ClaiError.
- **File size:** Truncate/recreate if file exceeds 10 MB.

---

## Part 4: Configuration (Full Specification)

### 4.1 Precedence (Highest to Lowest)

1. **CLI flags** (e.g. `--provider`, `--model`, `-i`, `-n`, `-f`, `-o`, `-q`, `-v`, `--no-color`, `--color`, `--debug`, `--debug-file`).
2. **Environment variables** (e.g. `CLAI_PROVIDER_DEFAULT`, `CLAI_CONTEXT_MAX_FILES` — see list below).
3. **File config** (merged from multiple paths, see below).
4. **Defaults** (in code).

### 4.2 Config File Locations (Order of Precedence, Highest First)

1. `./.clai.toml` (current directory)
2. `$XDG_CONFIG_HOME/clai/config.toml` (if `XDG_CONFIG_HOME` set)
3. `~/.config/clai/config.toml` (platform-specific home, e.g. via `directories` crate)
4. `/etc/clai/config.toml`

Only **existing** paths are considered. Merging: start from defaults, then merge each file from **lowest** to **highest** priority so that higher-priority file wins. Then merge env, then CLI.

### 4.3 File Config Schema (TOML, kebab-case)

```toml
[provider]
default = "openrouter"        # default provider name
fallback = ["ollama"]         # optional fallback list

[provider.openrouter]         # or [provider.ollama], etc.
api_key = "sk-..."            # optional; prefer env
api_key_env = "OPENROUTER_API_KEY"  # optional env var name
model = "qwen/qwen3-coder"    # optional default model
endpoint = null               # for local providers (e.g. Ollama)

[context]
max-files = 10
max-history = 3
redact-paths = false
redact-username = false

[safety]
confirm-dangerous = true
dangerous-patterns = [         # regex list; if empty, use built-in defaults
  "rm\\s+-rf\\s+/",
  "dd\\s+if=/dev/zero",
  # ...
]

[ui]
color = "auto"                # "auto" | "always" | "never"
debug-log-file = null         # e.g. "~/cache/clai/debug.log"
interactive = false
```

**Provider API key resolution (OpenRouter):**  
1) `[provider.openrouter].api_key`; 2) env var named in `api_key_env`; 3) `OPENROUTER_API_KEY`. If config has `${VAR}` or `$VAR`, loader can resolve from env (see `resolve_env_var_reference`).

### 4.4 File Config Security (Unix)

- **Permissions:** On Unix, config file must be **0600** (read/write owner only). Otherwise load fails with `InsecurePermissions`.
- **Missing file:** Not an error; use defaults.
- **Parse error / permission error:** Fatal; exit code 3 (Config).

### 4.5 Environment Variables (CLAI_* and Others)

**Merger reads (CLAI_* lowercased):**
- `CLAI_PROVIDER_DEFAULT`, `CLAI_PROVIDER_FALLBACK` (comma-separated)
- `CLAI_CONTEXT_MAX_FILES`, `CLAI_CONTEXT_MAX_HISTORY`, `CLAI_CONTEXT_REDACT_PATHS`, `CLAI_CONTEXT_REDACT_USERNAME`
- `CLAI_SAFETY_DANGEROUS_PATTERNS` (comma-separated), `CLAI_SAFETY_CONFIRM_DANGEROUS`
- `CLAI_UI_COLOR`

**Other env:**
- `OPENROUTER_API_KEY` — used when no api_key/api_key_env in provider config.
- `NO_COLOR` — disable color (no-color.org).
- `CLICOLOR=0` / `CLICOLOR=1` — disable/enable color.
- `TERM=dumb` — disable color.
- `XDG_CONFIG_HOME` — config path.
- `HOME`, `USER` / `USERNAME`, `SHELL` — context and path redaction.

### 4.6 Runtime Config (from CLI Only in Practice)

- `instruction`, `model`, `provider`, `quiet`, `verbose`, `no_color`, `color`, `interactive`, `force`, `dry_run`, `context` (optional file path), `offline`, `num_options` (1–10, clamped), `debug`, `debug_log_file`.
- **Interactive:** If file config has `ui.interactive = true` and CLI did not set `-i`, runtime still sets `interactive = true` (CLI can only add, not remove, interactive).
- **Debug log path:** CLI `--debug-file` wins; else file config `ui.debug_log_file` (then expand `~`).

### 4.7 Config Cache

- First call to `get_file_config(cli)` runs full merge (paths → load → env → CLI) and caches result in a global. Subsequent calls return cached result (same CLI shape in practice). Used for tests/benchmarks: reset cache when testing.

---

## Part 5: OpenRouter Integration (Technical)

### 5.1 API

- **URL:** `https://openrouter.ai/api/v1/chat/completions`
- **Method:** POST, JSON body.
- **Timeout:** 60 seconds (reqwest in Rust).

### 5.2 Request Body

- `model`: string (required).
- `messages`: array of `{ role, content }` (roles: system, user, assistant).
- `temperature`: optional (omit if not set).
- `max_tokens`: optional (omit if not set).

### 5.3 Response Handling

- **2xx:** Parse JSON; take `choices[0].message.content`; map usage if present.
- **4xx/5xx:** Error message includes status code; body included in message. Log to file logger if present.
- **429:** Retry with exponential backoff (1s, 2s, 4s), up to 3 attempts.
- **401/403:** “Authentication error (status): …”
- **408/504:** “Timeout error (status): …”
- Network/parse errors: logged; error propagated with context.

### 5.4 Model Selection

- Order: request’s `model` → provider’s default_model (from config) → `qwen/qwen3-coder`.
- CLI `--model` can be `modelId` or `provider/modelId`; chain’s `parse_model` splits on first `/`; if no `/`, use default provider.

### 5.5 Provider Chain

- **Order:** default provider first, then `fallback` list (no duplicates).
- **Lazy init:** Each provider (e.g. OpenRouter) is created on first use and cached.
- **Availability:** OpenRouter “available” if API key is non-empty.
- **Failure:** If one provider fails, try next; if all fail, return last error.

---

## Part 6: Context Gathering

### 6.1 Inputs

- **Config:** From `get_file_config(cli)` (context section + redaction flags).
- **CWD:** From `process.cwd()` (or equivalent); fail if unavailable.

### 6.2 System Context

- **Source:** sysinfo (OS name, OS version), `ARCH`, `SHELL`, `USER` or `USERNAME`, total memory.
- **Output:** Map: `os_name`, `os_version`, `architecture`, `shell`, `user`, `total_memory_mb`.
- **Caching:** Per run (e.g. static cache); same for all calls in same process.

### 6.3 Directory Context

- **Scope:** Current working directory only.
- **Limit:** `context.max_files` (default 10); sort by filename, take first N.
- **Truncation:** Paths longer than 80 chars replaced by basename.
- **Redaction:** If `context.redact_paths`: replace `$HOME`, `~/`, `$HOME/`, `/home/<user>/` with `[REDACTED]` (or equivalent).
- **Output:** List of path strings (and cwd); cwd also redacted if redact_paths.

### 6.4 Shell History

- **Shell:** From `SHELL` (e.g. bash, zsh, fish).
- **Paths:** bash → `~/.bash_history`, zsh → `~/.zsh_history`, fish → `~/.local/share/fish/fish_history`; else no history.
- **Read:** Last N lines (N = `context.max_history`, default 3). Efficient “tail”: e.g. seek to end minus 4096 bytes, then read lines and take last N.
- **Missing/error:** Return empty list.

### 6.5 Stdin

- **When:** Only when stdin is **not** a TTY (i.e. piped).
- **Limit:** 10 KB; truncate if longer.
- **Encoding:** UTF-8; invalid sequences replaced (e.g. lossy decode).
- **Empty pipe:** Treat as empty string (still include “Stdin input:” in prompt if you want consistency).

### 6.6 Optional Context File

- CLI `-c` / `--context` can pass a path; current Rust does not read this file into the prompt in the reviewed code; you may define in TypeScript whether to append file contents to the prompt.

### 6.7 Output Format

- Context is assembled into a JSON-like structure: `system`, `cwd`, `files`, `history`, `stdin`.
- This is then formatted into the **user message** string (sections “System Context”, “Directory Context”, “Recent Shell History”, “User Instruction”, plus instruction line).

---

## Part 7: Safety (Dangerous Commands)

### 7.1 When a Command Is “Dangerous”

- A command string is dangerous if it **matches any** of the compiled regexes from `safety.dangerous_patterns` (or built-in defaults if list is empty).

### 7.2 Default Patterns (Regex)

- `rm\s+-rf\s+/`
- `rm\s+-rf\s+/\s*$`
- `dd\s+if=/dev/zero`
- `mkfs\.\w+\s+/dev/`
- `sudo\s+rm\s+-rf\s+/`
- `>\s*/dev/`
- `format\s+[c-z]:`
- `del\s+/f\s+/s\s+[c-z]:\\`

Invalid regex in config: compilation fails and is fatal (or fail-safe: treat as dangerous in detector).

### 7.3 When to Prompt for Dangerous Command

- **Conditions:** (1) Command is dangerous, (2) `safety.confirm_dangerous` is true, (3) not `--force`, (4) stdin and stdout are TTY.
- If any condition fails, **do not** prompt; just print the (first) command to stdout (so piping still works).

### 7.4 Dangerous Prompt Behavior

- Print to **stderr:** “⚠️  DANGEROUS: <command>” (optionally colored).
- Print to stderr: “[E]xecute/[C]opy/[A]bort? ”.
- Read one line from stdin; first non-whitespace character (case-insensitive):
  - **E** → Execute: print command to stdout (for piping or display).
  - **C** → Copy: same as Execute in current Rust (print to stdout).
  - **A** → Abort: do not print command; exit with Safety error (exit code 5).
- EOF / pipe / empty input → treat as Abort.
- Invalid character → ConfirmationError; in main, treat as Abort (Safety).

### 7.5 Interactive Mode (Safe Commands, Multiple Options)

- When **not** dangerous and **interactive** (TTY + `-i` or config) and **multiple commands** (`-o N` with N>1):
  - Show prompt: “❯ <command> [1/N] (Tab: next | Enter | ^C)” on stderr.
  - **Tab:** cycle to next command (replace line in place; handle wrapped lines by clearing last N lines).
  - **Enter:** execute selected command via shell (e.g. `sh -c "<command>"`); exit code = command exit code (or 1 on failure).
  - **Esc / Ctrl+C:** abort (Safety, exit 5).
- When only one command: same prompt but “Enter | ^C” only; Enter = execute.
- If not TTY: do not prompt; return (Output, first command) and main prints first command to stdout.
- **Execute** in code: run shell with the selected command; capture exit code; if non-zero, main can report as General error (exit 1).

### 7.6 Dry-Run

- `-n` / `--dry-run`: **Always** print all generated commands to stdout (one per line if multiple), then exit. No safety prompt, no execute. Trailing newline for readability when TTY.

### 7.7 Single Command, Non-Interactive, Non-Dangerous

- Print first (and only) command to stdout (newline only if TTY).

---

## Part 8: CLI Reference

### 8.1 Arguments

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| instruction | (positional) | string | required | Natural language instruction. |
| --model | -m | string | — | Override model. |
| --provider | -p | string | — | Override provider. |
| --quiet | -q | flag | false | Minimal output. |
| --verbose | -v | count | 0 | Increase verbosity (e.g. 1 = info, 2 = debug). |
| --no-color | — | flag | false | Disable color. |
| --color | — | auto\|always\|never | auto | Color mode. |
| --interactive | -i | flag | false | Interactive (prompt execute/output/abort). |
| --force | -f | flag | false | Skip dangerous confirmation. |
| --dry-run | -n | flag | false | Only print command(s), no execute/prompt. |
| --context | -c | string | — | Optional context file path. |
| --offline | — | flag | false | Reserved; currently returns error. |
| --options | -o | number | 3 | Number of command options (1–10). |
| --debug | -d | flag | false | Print prompt/request to stderr. |
| --debug-file | — | optional path | — | Enable file logging (default path if value empty). |

### 8.2 Help and Version

- `--help` / `-h`: Print help to **stdout**; exit 0.
- `--version` / `-V`: Print version to **stdout**; exit 0.
- Parsing errors (e.g. missing required): **Usage** error to stderr; exit 2.

---

## Part 9: Error Handling and Exit Codes

### 9.1 Exit Codes

| Code | Meaning |
|------|--------|
| 0 | Success; or help/version. |
| 1 | General error (e.g. execute failed, interrupt). |
| 2 | Usage (invalid/missing CLI args). |
| 3 | Configuration (parse, permissions, invalid regex in safety). |
| 4 | API (network, auth, rate limit, parse response). |
| 5 | Safety (user abort or confirmation error treated as abort). |
| 130 | Interrupted (SIGINT/SIGTERM). |

### 9.2 Error Reporting

- **stderr:** Human-readable message; if verbose ≥ 1, append “backtrace” (cause chain).
- **File log:** If debug file enabled, log structured error (event + message + context).
- **API status:** If error message contains “(401)” or “(429)” etc., parse and expose for exit code 4 (and optional retries/logging).

### 9.3 Interruption

- Register SIGINT and SIGTERM; set a shared “interrupted” flag.
- Before and after main logic, check flag; if set, exit 130 (or 1, depending on convention).
- SIGPIPE: ignore (no handler) so pipe break doesn’t kill process.

---

## Part 10: Output and Logging

### 10.1 Print Command

- **Piped stdout:** Print command only, **no** trailing newline (so `clai "..." | sh` works).
- **TTY stdout:** Print command with trailing newline.
- Multiple commands (dry-run): print each; blank line between; trailing newline at end when TTY.

### 10.2 Logger (Stderr)

- Built from runtime config: quiet → Error level; else verbosity 0→Warning, 1→Info, 2→Debug, 3→Trace.
- Color: only if color mode says yes and stderr is TTY; prefix by level (ERROR/WARN/INFO/DEBUG/TRACE).

### 10.3 File Logger

- JSONL; one object per line.
- Fields: `ts`, `level`, `event`, plus event payload.
- Max file size 10 MB; truncate (e.g. delete and recreate) when opening.
- Create parent directories if needed.

---

## Part 11: Signals and TTY

### 11.1 TTY / Pipe

- **stdin TTY:** required for reading confirmation and interactive prompts.
- **stdout TTY:** used to decide newline after command and “interactive” display.
- **stderr TTY:** used for color and for showing prompts.
- **is_interactive:** stdin and stdout both TTY.
- **is_piped:** stdout not TTY.

### 11.2 Signals

- **SIGINT (Ctrl+C):** set interrupted; exit 130.
- **SIGTERM:** set interrupted; exit 130.
- **SIGPIPE:** do not handle (ignore) so that broken pipe does not kill the process.

---

## Part 12: Edge Cases and Behaviors (Checklist for Port)

### 12.1 Config

- [ ] No config file → use defaults; do not fail.
- [ ] Multiple files → merge in precedence order; higher priority wins.
- [ ] File not found in list → skip (non-fatal).
- [ ] Parse error or permission error in any file → fatal, exit 3.
- [ ] Unix: require 0600 on config file.
- [ ] Env vars CLAI_* override file; CLI overrides env.
- [ ] `num_options` clamped 1–10.
- [ ] `--no-color` overrides `--color`.
- [ ] Interactive: file config can enable; CLI `-i` can enable; do not allow CLI to disable if file set.
- [ ] Debug log path: CLI then file config; expand `~`.

### 12.2 Context

- [ ] CWD failure → fatal (context gathering error).
- [ ] Directory read error → empty file list.
- [ ] History file missing or unreadable → empty history.
- [ ] Stdin only read when not TTY; max 10 KB; UTF-8 lossy.
- [ ] Redaction: paths and optionally username; apply to cwd and file list.
- [ ] System info cached for process lifetime.

### 12.3 AI / OpenRouter

- [ ] Model: CLI → provider config → default `qwen/qwen3-coder`.
- [ ] API key: config api_key → api_key_env → OPENROUTER_API_KEY.
- [ ] 429 → retry with backoff (e.g. 3 times).
- [ ] Log request/response to file logger when enabled.
- [ ] Single command: strip ```bash / ```sh / ```shell / ``` fences; trim; fallback = full trim.
- [ ] Multi-command: parse JSON `{"commands": [...]}`; strip ```json / ``` if present; fallback to raw array; then to single command; empty array or all empty strings → error.

### 12.4 Safety

- [ ] Dangerous = any regex match.
- [ ] Invalid regex in config → fail (or fail-safe: treat as dangerous).
- [ ] Prompt only when: dangerous + confirm_dangerous + !force + TTY (stdin + stdout).
- [ ] EOF / invalid input on dangerous prompt → Abort (exit 5).
- [ ] Dry-run: no safety check; print all commands.
- [ ] Interactive (safe, multi): Tab cycles; Enter executes; Esc/Ctrl+C aborts.
- [ ] Execute: shell `-c "<command>"`; return exit code; non-zero → General error (1).

### 12.5 Output and Flow

- [ ] stdout = command only (and optional newline when TTY).
- [ ] Dry-run with multiple: each command on its own line; blank between; newline at end if TTY.
- [ ] Help/version → stdout, exit 0.
- [ ] All other errors → stderr; exit per ClaiError code.
- [ ] Interrupt check before and after heavy work; exit 130 if set.

### 12.6 Locale and Color

- [ ] Color: NO_COLOR, CLICOLOR=0/1, TERM=dumb, stderr TTY.
- [ ] Locale: LANG for future use; language code extraction (e.g. en, fr).

---

## Part 13: TypeScript Port Notes

### 13.1 Suggested Layout

- **CLI:** e.g. `commander` or `yargs`; define options to match table above.
- **Config:** TOML parser (e.g. `@iarna/toml` or `toml`); same schema and precedence; implement merger and cache (sync or async as needed).
- **Context:** `process.cwd()`, `fs.readdirSync`/`fs.promises`, `os`/`process.env` for system and shell; tail history file (last N lines); stdin only when `!process.stdin.isTTY`, with max bytes.
- **HTTP:** `fetch` or `axios` to OpenRouter; same headers and body; retry on 429.
- **Safety:** `RegExp` or `new RegExp(pattern)` for each pattern; validate patterns at load time; match on trimmed command.
- **Interactive:** Use a library (e.g. `inquirer`, `@inquirer/prompts`) for E/C/A and for Tab/Enter/Esc; or raw TTY read (e.g. `readline` + raw mode).
- **Execute:** `child_process.spawnSync` or `execSync` with shell (e.g. `process.platform === 'win32' ? 'cmd' : process.env.SHELL || '/bin/sh'`, `-c`, command).
- **Signals:** `process.on('SIGINT', ...)`, `process.on('SIGTERM', ...)`; set flag and exit with code 130.
- **Exit:** `process.exit(code)` with codes from table.

### 13.2 Consistency with Rust

- Preserve stdout/stderr semantics so that `clai "..." | sh` and scripts behave the same.
- Preserve exit codes so that callers can distinguish usage, config, API, safety.
- Preserve config precedence and file locations so that existing configs work.
- Preserve dangerous pattern semantics (regex, prompt conditions) so that safety guarantees are unchanged.

### 13.3 Optional Enhancements (Out of Scope for Rust Parity)

- Clipboard copy for “Copy” (e.g. `clipboardy` on Node).
- Reading `-c`/`--context` file into the prompt.
- Local provider (e.g. Ollama) and offline mode.
- Streaming AI response (if OpenRouter supports it).

---

**End of document.** Use this as the single source of truth for the TypeScript port: PRD (Part 1), architecture and data flow (Parts 2–3), and full technical specs (Parts 4–12), with port checklist and notes (Parts 12–13).
