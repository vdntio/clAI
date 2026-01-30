#!/usr/bin/env node
// clai - CLI for converting natural language to shell commands

import { parseCli } from './cli/index.js'
import { getConfig, ConfigError } from './config/index.js'
import { gatherContext, ContextError } from './context/index.js'
import {
  generateCommands,
  AIError,
  buildPrompt,
  formatPromptForDebug,
} from './ai/index.js'
import { checkSafety, SafetyError } from './safety/index.js'
import {
  renderUI,
  UserAction,
  withSpinner,
  printCommand,
  printWarning,
} from './ui/index.js'
import { executeCommand, ExecutionError } from './output/index.js'
import { UsageError, InterruptError } from './error/index.js'

async function main(): Promise<void> {
  try {
    // Parse CLI arguments
    const cli = parseCli(process.argv)

    // Load and merge config (file + env + CLI)
    const config = getConfig(cli)

    // Handle offline mode (not yet implemented)
    if (config.offline) {
      process.stderr.write('Error: Offline mode is not yet supported\n')
      process.exit(1)
    }

    // Gather context for AI prompt
    const context = await gatherContext(config)

    // Debug output
    if (config.debug) {
      process.stderr.write('=== Debug: Loaded Config ===\n')
      process.stderr.write(`Provider: ${config.provider.default}\n`)
      process.stderr.write(`Context maxFiles: ${config.context.maxFiles}\n`)
      process.stderr.write(`Context maxHistory: ${config.context.maxHistory}\n`)
      process.stderr.write(
        `Safety confirmDangerous: ${config.safety.confirmDangerous}\n`
      )
      process.stderr.write(`UI color: ${config.ui.color}\n`)
      process.stderr.write(`UI interactive: ${config.ui.interactive}\n`)
      process.stderr.write(`UI numOptions: ${config.ui.numOptions}\n`)
      process.stderr.write(`Model: ${config.model || '(default)'}\n`)
      process.stderr.write(`Verbose: ${config.verbose}\n`)
      process.stderr.write(`Dry run: ${config.dryRun}\n`)
      process.stderr.write(`Force: ${config.force}\n`)
      process.stderr.write(`===========================\n\n`)

      process.stderr.write('=== Debug: Gathered Context ===\n')
      process.stderr.write(
        `OS: ${context.system.osName} ${context.system.osVersion}\n`
      )
      process.stderr.write(`Architecture: ${context.system.architecture}\n`)
      process.stderr.write(`Shell: ${context.system.shell}\n`)
      process.stderr.write(`User: ${context.system.user}\n`)
      process.stderr.write(`Memory: ${context.system.totalMemoryMb} MB\n`)
      process.stderr.write(`CWD: ${context.cwd}\n`)
      process.stderr.write(`Files (${context.files.length}):\n`)
      context.files
        .slice(0, 5)
        .forEach((f) => process.stderr.write(`  - ${f}\n`))
      if (context.files.length > 5) {
        process.stderr.write(`  ... and ${context.files.length - 5} more\n`)
      }
      process.stderr.write(`History (${context.history.length}):\n`)
      context.history.forEach((h) => process.stderr.write(`  - ${h}\n`))
      if (context.stdin) {
        process.stderr.write(
          `Stdin: ${context.stdin.substring(0, 100)}${context.stdin.length > 100 ? '...' : ''}\n`
        )
      } else {
        process.stderr.write(`Stdin: (none - not piped)\n`)
      }
      process.stderr.write(`===============================\n\n`)

      // Show the full prompt being sent to AI
      const messages = buildPrompt(
        context,
        config.instruction,
        config.ui.numOptions
      )
      process.stderr.write('=== Debug: AI Prompt ===\n')
      process.stderr.write(formatPromptForDebug(messages))
      process.stderr.write(`\n========================\n\n`)
    }

    // Generate commands from AI (with spinner)
    const commands = await withSpinner(
      'Thinking...',
      () => generateCommands(context, config.instruction, config)
    )

    // Output the generated commands
    if (config.dryRun) {
      // Dry-run: show all commands with comments
      process.stdout.write(`# Generated ${commands.length} command(s):\n`)
      commands.forEach((cmd, i) => {
        process.stdout.write(`# Option ${i + 1}:\n${cmd}\n\n`)
      })
      process.exit(0)
    }

    // Check safety of generated commands
    const safety = checkSafety(commands, config)

    if (config.debug) {
      process.stderr.write('=== Debug: Safety Check ===\n')
      process.stderr.write(`Is dangerous: ${safety.isDangerous}\n`)
      process.stderr.write(`Should prompt: ${safety.shouldPrompt}\n`)
      process.stderr.write(`===========================\n\n`)
    }

    // Determine if we should show interactive UI
    // Always show in TTY mode (we're executing, not just copying)
    // Skip only if: piped, force flag, or dry-run
    const isTTY = process.stdin.isTTY && process.stdout.isTTY
    const showUI = isTTY && !config.force

    let selectedCommand: string

    if (showUI) {
      // Show interactive UI for command selection
      const result = await renderUI({
        commands,
        config,
        isDangerous: safety.isDangerous,
      })

      if (result.action === UserAction.Abort) {
        throw new SafetyError('Command execution aborted by user')
      }

      selectedCommand = result.command
    } else {
      // Non-interactive: use first command
      selectedCommand = commands[0] ?? ''

      // Show warning for dangerous commands in non-interactive mode
      if (safety.isDangerous && !config.force) {
        printWarning('This command may be dangerous. Use -f to skip this warning.')
      }
    }

    // Execute or output the selected command
    if (selectedCommand) {
      if (showUI) {
        // Interactive: execute the command
        const result = await executeCommand(selectedCommand)

        if (!result.success) {
          process.stderr.write(`Error: ${result.error.message}\n`)
          process.exit(result.error.code)
        }

        process.exit(result.exitCode)
      } else {
        // Non-interactive (piped): just output the command
        printCommand(selectedCommand, safety.isDangerous)
        process.exit(0)
      }
    } else {
      process.exit(0)
    }
  } catch (error) {
    if (error instanceof UsageError) {
      process.stderr.write(`Error: ${error.message}\n`)
      process.stderr.write("Try 'clai --help' for more information.\n")
      process.exit(error.code)
    }

    if (error instanceof ConfigError) {
      process.stderr.write(`Config error: ${error.message}\n`)
      process.exit(error.code)
    }

    if (error instanceof ContextError) {
      process.stderr.write(`Context error: ${error.message}\n`)
      process.exit(error.code)
    }

    if (error instanceof AIError) {
      process.stderr.write(`AI error: ${error.message}\n`)
      process.exit(error.code)
    }

    if (error instanceof SafetyError) {
      process.stderr.write(`Aborted: ${error.message}\n`)
      process.exit(error.code)
    }

    if (error instanceof ExecutionError) {
      process.stderr.write(`Execution error: ${error.message}\n`)
      process.exit(error.code)
    }

    if (error instanceof InterruptError) {
      process.stderr.write(`\n${error.message}\n`)
      process.exit(error.code)
    }

    if (error instanceof Error) {
      process.stderr.write(`Error: ${error.message}\n`)
      process.exit(1)
    }

    process.exit(1)
  }
}

main()
