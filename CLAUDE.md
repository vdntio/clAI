# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**clai** is a CLI that converts natural language instructions into executable shell commands using an AI provider (OpenRouter). Currently in **alpha release (v0.1.0-alpha.1)**.

Example usage:
```bash
clai "find all typescript files modified today"
# outputs: find . -name "*.ts" -mtime 0
```

## Distribution

- **npm**: Published as `clai@alpha` (install: `npm install -g clai@alpha`)
- **Binaries**: GitHub Releases provides standalone binaries for Linux/macOS/Windows
- **GitHub**: https://github.com/vdntio/clAI

## Architecture

```
CLI parse → Config build → Context gathering → Prompt build → AI (OpenRouter) → Safety check → Output
```

**Core modules to implement:**
- `cli` - Argument parsing (commander or yargs)
- `config` - TOML config loading, env vars, CLI merge (precedence: CLI > env > file > defaults)
- `context` - System info, cwd, directory scan, shell history, stdin
- `ai` - Prompt construction, OpenRouter API, response parsing
- `safety` - Dangerous command regex detection, E/C/A confirmation prompt
- `output` - stdout for commands only (no trailing newline when piped), stderr for everything else

## Critical Behaviors

**stdout/stderr separation:**
- stdout = generated command(s) ONLY
- stderr = logs, warnings, prompts, debug output, errors
- When piped: no trailing newline on stdout
- When TTY: trailing newline on stdout

**Exit codes:**
- 0: Success or help/version
- 1: General error
- 2: Usage (invalid CLI args)
- 3: Config error
- 4: API error
- 5: Safety (user abort)
- 130: Interrupted (SIGINT/SIGTERM)

**Config file locations (highest priority first):**
1. `./.clai.toml`
2. `$XDG_CONFIG_HOME/clai/config.toml`
3. `~/.config/clai/config.toml`
4. `/etc/clai/config.toml`

Unix: config files must be 0600 permissions.

## Tooling & Conventions

- **Package manager:** Use **bun** only (no npm). Commands: `bun init -y`, `bun add -d` for dev deps, `bun install`, `bun run build` / `bun run test` / `bun run lint` / `bun run dev`.
- **Task Master:** Tasks live in `.taskmaster/tasks/tasks.json`; PRD for task generation in `.taskmaster/docs/prd.txt`. MCP tools require `projectRoot` as absolute path. After `update_task`, re-run `expand_task` if a task lost subtasks.
- **Task Master workflow:** Mark tasks as `done` via `set_task_status` after completing implementation.
- **Vitest + CLI tests:** Set `test.disableConsoleIntercept: true` in `vitest.config.ts` so stdout/stderr capture works when asserting CLI output.

**Conventions & gotchas:**
- ESLint 9 flat config: use `ignores` in `eslint.config.js`, not `.eslintignore`.
- ESLint ignores: Add external/generated directories to `ignores` (e.g., `.opencode/**`).
- ESLint test files: Allow `@typescript-eslint/no-explicit-any` in `tests/**/*.ts` for mocking globals.
- Empty catch blocks: Add a comment (e.g., `// Directory may already exist`) to satisfy `no-empty` rule.
- Re-export pattern: Don't import-then-re-export from same module; use direct `export { X } from './types'`.
- TypeScript interface exports: Use `export type { Interface }` not `export { Interface }` to avoid runtime errors with bun/vitest.
- TypeScript override keyword: Use `override` on methods that override parent class methods (required with strict TypeScript).
- TypeScript type-only exports: Use `export type { Type }` for types to avoid "export not found" errors in compiled JS.
- TypeScript protected methods: Use `protected` (not `private`) for methods that subclasses need to access.
- Chalk color forcing: Create instance with explicit level: `new Chalk({ level: 3 })` for always-on, `new Chalk({ level: 0 })` for never.
- Node globals in ESLint: use `globals` package (e.g. `globals.node`) so `process`, `__dirname` are recognized.
- Ink + React: tsconfig needs `"jsx": "react-jsx"` and `include` with `**/*.tsx`.
- Bun init: creates `index.ts` by default; remove or point entry to real entry (e.g. `src/main.ts`).
- Commander counting option: use callback `(_, prev) => prev + 1` for verbose/count-style flags.
- Error classes: Use `Object.setPrototypeOf(this, ClassName.prototype)` for proper instanceof checks across inheritance.
- Readonly at runtime: Use `Object.defineProperty(this, 'prop', { value, writable: false })` with `readonly prop!: Type` declaration.
- Circular deps: Base error classes should not re-export derived classes; keep imports unidirectional.
- Vitest sequential tests: Use `describe.sequential()` for tests sharing file system resources (temp dirs) to avoid race conditions.
- Vitest test isolation: Use unique temp directories per test suite (`/tmp/project-suite-name`) to prevent parallel test conflicts.
- Vitest timing tests: Use lenient assertions like `expect([130, null]).toContain(code)` for timing-dependent exit codes.
- Vitest ESM mocking: Cannot spy on ESM module exports (e.g., `fs.appendFileSync`); use integration tests or test invalid paths instead.
- Vitest run command: `describe.sequential` requires `bun run test` (vitest script), doesn't work with `bun test` directly.

## Recommended Stack

- **commander** - CLI argument parsing
- **zod** - Config and response validation
- **ink** + **react** - Terminal UI with animations, Tab-cycling prompts, spinners
- **@iarna/toml** - TOML config file parsing
- Direct fetch to OpenRouter (no SDK needed)

**Why Ink:** We want a delightful terminal experience with smooth animations, Tab-cycling between command options, and visual feedback. Ink's React model makes state management clean for interactive prompts.

**Piped mode:** When stdout is not a TTY, bypass Ink and use raw `process.stdout.write()` for script compatibility.
