# Potential Packages for TUI Application Development

This document evaluates each npm package from the OpenCode `package.json` (root and workspace catalog). For each package we describe **what it does**, then **whether it’s helpful for TUI/CLI application development** (and why). At the end we list **recommended TUI/CLI packages** that OpenCode does not use but are commonly used for terminal UIs and interactive CLIs.

**Context:** OpenCode is an AI-powered terminal/development tool. “TUI” here means both (1) **terminal UIs** (interactive prompts, key handling, colored output) and (2) **CLI tooling** (args, config, stdout/stderr, scripting). Some packages serve the **web/desktop UI** (Solid, Vite) that may wrap or accompany the terminal experience; those are noted as “Web/Desktop UI” rather than “TUI” but are still relevant to the overall app.

---

## How to Read This Doc

- **Use for TUI/CLI:** Yes / No / Maybe  
- **Why:** Short reason tied to terminal UIs, interactive prompts, CLI output, or scripting.  
- **Category:** TUI/CLI · Web/Desktop UI · AI/API · Infra/Tooling · Data/Utils · Other  

---

## 1. Root Dependencies

| Package | What it does | Use for TUI/CLI? | Why | Category |
|--------|----------------|-------------------|-----|----------|
| **@aws-sdk/client-s3** | Official AWS SDK for S3 (upload, download, list, etc.). | **No** | Cloud storage only; no direct terminal UI or CLI flow. | Infra/Cloud |
| **@opencode-ai/plugin** | OpenCode-specific plugin (internal to the project). | **Maybe** | Only relevant if you’re extending OpenCode; not a generic TUI library. | Internal |
| **@opencode-ai/script** | OpenCode-specific script (internal). | **Maybe** | Same as above. | Internal |
| **@opencode-ai/sdk** | OpenCode SDK (internal). | **Maybe** | Same as above. | Internal |
| **typescript** | TypeScript compiler and type checker. | **Yes** | Type safety and structure for any CLI/TUI codebase. | Infra/Tooling |

---

## 2. Root DevDependencies

| Package | What it does | Use for TUI/CLI? | Why | Category |
|--------|----------------|-------------------|-----|----------|
| **@actions/artifact** | GitHub Actions artifact upload/download. | **No** | CI/CD only. | Infra/CI |
| **@tsconfig/bun** | Shared TSConfig for Bun. | **Yes** | Consistent TS settings; Bun can run CLI/TUI scripts. | Infra/Tooling |
| **husky** | Git hooks (pre-commit, etc.). | **No** | Developer workflow; doesn’t affect TUI behavior. | Infra/Tooling |
| **prettier** | Code formatter. | **No** | Formatting only. | Infra/Tooling |
| **semver** | Parse and compare semver versions. | **Maybe** | Useful if your CLI checks or displays versions. | Data/Utils |
| **sst** | SST (Serverless Stack) for IaC/deploy. | **No** | Deploy/infra; not TUI. | Infra/Cloud |
| **turbo** | Turborepo – monorepo task runner. | **No** | Builds/tasks; not TUI. | Infra/Tooling |

---

## 3. Workspace Catalog (alphabetical)

| Package | What it does | Use for TUI/CLI? | Why | Category |
|--------|----------------|-------------------|-----|----------|
| **@cloudflare/workers-types** | TypeScript types for Cloudflare Workers. | **No** | Edge runtime types; not terminal. | Infra/Tooling |
| **@hono/zod-validator** | Hono middleware: validate request body/query/etc. with Zod. | **No** | Web API validation; not CLI. | Web/API |
| **@kobalte/core** | Unstyled, accessible UI primitives for **SolidJS** (web). | **No** | Web/desktop UI; not terminal. | Web/Desktop UI |
| **@octokit/rest** | GitHub REST API client. | **Maybe** | Useful if your TUI/CLI talks to GitHub (e.g. auth, repos). | AI/API |
| **@openauthjs/openauth** | OpenAuth (auth library). | **Maybe** | If your CLI does OAuth or app auth. | AI/API |
| **@pierre/diffs** | Diff/patch utilities (likely text or structured diffs). | **Yes** | Showing diffs in terminal (e.g. “before/after” command or config). | TUI/CLI |
| **@playwright/test** | E2E testing (browser/electron). | **No** | Browser/desktop testing; not terminal E2E. | Infra/Tooling |
| **@solid-primitives/storage** | Persist Solid signals/stores (e.g. localStorage, sessionStorage). | **No** | Browser storage; not terminal. | Web/Desktop UI |
| **@solidjs/meta** | SolidJS head/meta (titles, etc.). | **No** | Web only. | Web/Desktop UI |
| **@solidjs/router** | Client-side routing for Solid. | **No** | Web only. | Web/Desktop UI |
| **@solidjs/start** | SolidStart – full-stack meta-framework for Solid. | **No** | Web/SSR; not TUI. | Web/Desktop UI |
| **@tailwindcss/vite** | Tailwind CSS plugin for Vite. | **No** | Web styling. | Web/Desktop UI |
| **@types/bun** | TypeScript types for Bun. | **Yes** | Bun as runtime for CLI/TUI scripts. | Infra/Tooling |
| **@types/luxon** | Types for Luxon. | **Maybe** | If CLI shows timestamps or relative time. | Data/Utils |
| **@types/node** | Node.js type definitions. | **Yes** | If using Node APIs (stdio, fs, process) in CLI. | Infra/Tooling |
| **@types/semver** | Types for semver. | **Maybe** | Version handling in CLI. | Data/Utils |
| **@tsconfig/node22** | Shared TSConfig for Node 22. | **Yes** | Consistent TS for Node-based CLI. | Infra/Tooling |
| **ai** | **Vercel AI SDK** – provider-agnostic AI (streaming, tools, multi-provider). | **Yes** | Call OpenAI/Anthropic/etc. from a CLI; streaming output in terminal. | TUI/CLI · AI/API |
| **diff** | Text diff (e.g. jsdiff – character/word/lines, patches). | **Yes** | Show diffs in terminal (config, prompts, or command output). | TUI/CLI |
| **dompurify** | Sanitize HTML to prevent XSS. | **Maybe** | If you render markdown/HTML in a **web** view; less relevant for pure terminal. | Web/Desktop UI |
| **fuzzysort** | Fast fuzzy search (e.g. filter lists by typo-tolerant query). | **Yes** | Search/filter in interactive CLI (commands, history, files). | TUI/CLI |
| **hono** | Small, fast web framework (multi-runtime: Node, Bun, Workers). | **Maybe** | If your “TUI” has a local API or server (e.g. OpenCode’s backend). | Web/API |
| **hono-openapi** | OpenAPI support for Hono. | **No** | API docs; not TUI. | Web/API |
| **luxon** | Date/time (IANA zones, formatting, parsing). | **Maybe** | Timestamps, “last run”, logs in CLI. | Data/Utils |
| **marked** | Markdown → HTML (fast, extensible). | **Yes** | Render AI/output or help as markdown; often paired with a sanitizer for web. | TUI/CLI · Web |
| **marked-shiki** | Marked extension: Shiki syntax highlighting in markdown code blocks. | **Yes** | Pretty-print code in terminal or in a web view fed by CLI. | TUI/CLI · Web |
| **remeda** | Functional utils (filter, map, pipe, etc.) with strong TypeScript inference. | **Yes** | Data transforms in CLI (config, responses, lists). | Data/Utils |
| **shiki** | Syntax highlighting (TextMate grammars → HTML/tokens). | **Yes** | Highlight code in CLI output or in a companion web UI. | TUI/CLI · Web |
| **solid-list** | Virtual list for Solid (long lists). | **No** | Solid/web only; terminal doesn’t use “virtual list” in the same way. | Web/Desktop UI |
| **solid-js** | Reactive UI framework (signals, no VDOM). | **No** | Web/desktop UI; OpenCode’s terminal UI may be separate. | Web/Desktop UI |
| **tailwindcss** | Utility CSS framework. | **No** | Web only. | Web/Desktop UI |
| **@typescript/native-preview** | Native (Go) TypeScript compiler preview (`tsgo`) – faster typecheck/build. | **Maybe** | Speeds up CLI/TS tooling in dev; experimental. | Infra/Tooling |
| **typescript** | TypeScript compiler. | **Yes** | Core tool for any TS CLI/TUI. | Infra/Tooling |
| **ulid** | Lexicographically sortable unique IDs. | **Maybe** | Session IDs, request IDs, or log correlation in CLI. | Data/Utils |
| **virtua** | Virtual scroll (React/Vue/Solid/Svelte). | **No** | Web/desktop lists; not terminal. | Web/Desktop UI |
| **vite** | Build tool and dev server. | **Maybe** | If you bundle a CLI or a web UI that accompanies the TUI. | Infra/Tooling |
| **vite-plugin-solid** | Vite plugin for Solid. | **No** | Web build only. | Web/Desktop UI |
| **zod** | Schema validation and parsing (TypeScript-first). | **Yes** | Validate CLI args, config files, env, API responses. | TUI/CLI |

---

## 4. Patched Dependencies

| Package | What it does | Use for TUI/CLI? | Why | Category |
|--------|----------------|-------------------|-----|----------|
| **ghostty-web** | Ghostty terminal (web/embed variant). | **Yes** | This *is* the terminal surface; patches likely fix integration or behavior. | TUI/CLI |

---

## 5. Package Manager / Runtime

| Tool | What it does | Use for TUI/CLI? | Why | Category |
|------|----------------|-------------------|-----|----------|
| **bun** (packageManager) | Runtime + package manager (fast install, run, test). | **Yes** | Run and script CLI/TUI; fast startup helps interactive tools. | Infra/Tooling |

---

## 6. Summary: High-Value for TUI/CLI

These are the ones from OpenCode’s stack that are **most directly useful** for TUI/CLI work:

- **ai** (Vercel AI SDK) – Call models, stream output, tools; fits “AI in the terminal.”
- **zod** – Validate config, env, and API responses.
- **diff** / **@pierre/diffs** – Show diffs in terminal.
- **marked** / **marked-shiki** / **shiki** – Markdown and code highlighting (terminal or web).
- **fuzzysort** – Fuzzy search in interactive lists (history, files, commands).
- **remeda** – Data transformation in CLI logic.
- **typescript**, **@types/node**, **@types/bun**, **@tsconfig/*** – Type safety and runtimes (Node/Bun).
- **ghostty-web** – The terminal UI itself (with project-specific patches).
- **bun** – Runtime and scripts for the app.

---

## 7. Recommended TUI/CLI Packages NOT in OpenCode

OpenCode is largely a **web/desktop app** (Solid, Vite, Ghostty) with an **embedded terminal**. For a **standalone Node/Bun CLI** with prompts, key handling, and colored output (e.g. a tool like the clai port), these packages are commonly used and are **not** in OpenCode’s `package.json`:

### 7.1 Interactive prompts (TUI-style)

| Package | What it does | When to use |
|--------|----------------|-------------|
| **@inquirer/prompts** or **inquirer** | Prompts: input, select, confirm, list, etc. | When you need E/C/A, “choose one of N”, or form-like flows in the terminal. |
| **@inquirer/core** | Low-level hooks (useKeypress, useRef, etc.) for custom prompts. | When you want full control over key handling and layout (e.g. Tab cycling). |

### 7.2 Terminal UI (React-style or low-level)

| Package | What it does | When to use |
|--------|----------------|-------------|
| **ink** | React renderer for the terminal (Flexbox, components, hooks). | When you want a “React for CLI” with layout and `useInput`-style key handling. |
| **blessed** or **blessed-contrib** | Curses-like terminal UI (widgets, screen, input). | When you need low-level control and don’t want React. |

### 7.3 Terminal output (colors, styling)

| Package | What it does | When to use |
|--------|----------------|-------------|
| **chalk** | ANSI colors and styles (chainable API); respects TTY/NO_COLOR. | Most common; rich API and ecosystem. |
| **picocolors** | Tiny, fast ANSI colors; NO_COLOR friendly. | When you want minimal deps and maximum speed. |
| **kleur** | Lightweight colors; tree-shakable. | Middle ground between chalk and picocolors. |

### 7.4 Argument parsing and CLI structure

| Package | What it does | When to use |
|--------|----------------|-------------|
| **commander** | Define commands, options, and help. | Standard for structured CLIs. |
| **yargs** | Args parsing with builder API and plugins. | When you need complex CLI shapes. |
| **clipanion** | Type-safe CLI framework (used by Yarn). | When you want strong typing and composable commands. |

### 7.5 Terminal capabilities

| Package | What it does | When to use |
|--------|----------------|-------------|
| **readline** (Node built-in) | Line editing and key events. | When you only need simple line input. |
| **node-pty** | Pseudo-terminal (PTY); spawn shells and terminal processes. | When you embed a real shell (e.g. terminal tab). |
| **strip-ansi** | Remove ANSI codes from strings. | When measuring width or testing output. |
| **wrap-ansi** / **cli-width** | Wrap text to terminal width; get columns. | When you need to wrap or layout text. |

### 7.6 Suggested minimal set for a “clai-like” CLI

For a **TypeScript CLI** that does interactive prompts (E/C/A, Tab cycle), colored stderr, and clean stdout:

- **commander** or **yargs** – CLI structure.
- **zod** – Config and response validation (you already have this in catalog).
- **chalk** or **picocolors** – Colored stderr and prompts.
- **@inquirer/prompts** or **@inquirer/core** – Dangerous confirmation and “choose command” (or custom key handling with **readline** / **process.stdin**).
- **ai** (Vercel AI SDK) – If you want a unified AI layer; otherwise use **fetch** + OpenRouter directly.

If the “TUI” is a **web-based terminal** (like OpenCode with Ghostty), then **Solid + Vite + virtua/solid-list** and **shiki/marked** stay relevant for the web UI; the list in §7 is for the **terminal-side** behavior (e.g. scripts that run inside that terminal).

---

## 8. References

- Inquirer: [npmjs.com/package/inquirer](https://www.npmjs.com/package/inquirer), [github.com/SBoudrias/Inquirer.js](https://github.com/SBoudrias/Inquirer.js)
- Ink: [npmjs.com/package/ink](https://www.npmjs.com/package/ink)
- Chalk: [github.com/chalk/chalk](https://github.com/chalk/chalk)
- Picocolors: [github.com/alexeyraspopov/picocolors](https://github.com/alexeyraspopov/picocolors)
- Commander: [github.com/tj/commander.js](https://github.com/tj/commander.js)
- Vercel AI SDK: [sdk.vercel.ai](https://sdk.vercel.ai/docs), [npmjs.com/package/ai](https://www.npmjs.com/package/ai)
- Zod: [zod.dev](https://zod.dev)
- OpenCode: [opencode.ai](https://opencode.ai/download), [github.com/anomalyco/opencode](https://github.com/anomalyco/opencode)

---

*Document generated for evaluating OpenCode’s packages for TUI/CLI development and for choosing additional packages for terminal-based or hybrid terminal + web applications.*
