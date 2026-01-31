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
import { registerSignalHandlers, checkInterrupt } from './signals/index.js'
import { CombinedLogger, LogLevel } from './logging/index.js'

async function main(): Promise<void> {
  try {
    // Register signal handlers first
    registerSignalHandlers()

    // Parse CLI arguments
    const cli = parseCli(process.argv)

    // Load and merge config (file + env + CLI)
    const config = getConfig(cli)

    // Determine log level
    const logLevel: LogLevel = config.quiet
      ? 'quiet'
      : config.debug || config.verbose > 0
        ? 'verbose'
        : 'normal'

    // Create logger
    const logger = new CombinedLogger(logLevel, config.ui.color, config.debugFile)

    // Handle offline mode (not yet implemented)
    if (config.offline) {
      logger.error('Offline mode is not yet supported')
      process.exit(1)
    }

    // Check for interrupts before context gathering
    checkInterrupt()

    // Gather context for AI prompt
    const context = await gatherContext(config)

    // Debug output
    if (config.debug) {
      logger.debug('=== Loaded Config ===')
      logger.debug(`Provider: ${config.provider.default}`)
      logger.debug(`Context maxFiles: ${config.context.maxFiles}`)
      logger.debug(`Context maxHistory: ${config.context.maxHistory}`)
      logger.debug(`Safety confirmDangerous: ${config.safety.confirmDangerous}`)
      logger.debug(`UI color: ${config.ui.color}`)
      logger.debug(`UI interactive: ${config.ui.interactive}`)
      logger.debug(`UI numOptions: ${config.ui.numOptions}`)
      logger.debug(`Model: ${config.model || '(default)'}`)
      logger.debug(`Verbose: ${config.verbose}`)
      logger.debug(`Dry run: ${config.dryRun}`)
      logger.debug(`Force: ${config.force}`)
      logger.debug('===========================')

      logger.debug('=== Gathered Context ===')
      logger.debug(`OS: ${context.system.osName} ${context.system.osVersion}`)
      logger.debug(`Architecture: ${context.system.architecture}`)
      logger.debug(`Shell: ${context.system.shell}`)
      logger.debug(`User: ${context.system.user}`)
      logger.debug(`Memory: ${context.system.totalMemoryMb} MB`)
      logger.debug(`CWD: ${context.cwd}`)
      logger.debug(`Files (${context.files.length}):`)
      context.files
        .slice(0, 5)
        .forEach((f) => logger.debug(`  - ${f}`))
      if (context.files.length > 5) {
        logger.debug(`  ... and ${context.files.length - 5} more`)
      }
      logger.debug(`History (${context.history.length}):`)
      context.history.forEach((h) => logger.debug(`  - ${h}`))
      if (context.stdin) {
        logger.debug(
          `Stdin: ${context.stdin.substring(0, 100)}${context.stdin.length > 100 ? '...' : ''}`
        )
      } else {
        logger.debug(`Stdin: (none - not piped)`)
      }
      logger.debug('===============================')

      // Show the full prompt being sent to AI
      const messages = buildPrompt(
        context,
        config.instruction,
        config.ui.numOptions
      )
      logger.debug('=== AI Prompt ===')
      logger.debug(formatPromptForDebug(messages))
      logger.debug('========================')
    }

    // Check for interrupts before AI generation
    checkInterrupt()

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
      logger.debug('=== Safety Check ===')
      logger.debug(`Is dangerous: ${safety.isDangerous}`)
      logger.debug(`Should prompt: ${safety.shouldPrompt}`)
      logger.debug('===========================')
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

      // Check for interrupts after UI interaction
      checkInterrupt()
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
      // Check for interrupts before command execution
      checkInterrupt()

      if (showUI) {
        // Interactive: execute the command
        const result = await executeCommand(selectedCommand)

        if (!result.success) {
          logger.error(result.error.message)
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
    // Note: logger may not be available if error occurs before config loading
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
