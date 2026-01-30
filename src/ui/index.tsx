// src/ui/index.tsx
// Main entry point for the interactive UI module

import React from 'react'
import { render } from 'ink'
import { App } from './App.js'
import {
  UserAction,
  UIPhase,
  type RenderOptions,
  type RenderResult,
} from './types.js'

// Re-export types and enums
export { UserAction, UIPhase } from './types.js'
export type {
  RenderOptions,
  RenderResult,
  UIState,
  AppProps,
  TerminalSize,
} from './types.js'

// Re-export hooks
export { useTimeout } from './hooks/useTimeout.js'
export { useTerminalSize } from './hooks/useTerminalSize.js'
export { useAnimation, usePulse, useTypewriter } from './hooks/useAnimation.js'

// Re-export utilities
export {
  formatCommand,
  formatCounter,
  getCommandDisplayWidth,
  truncateMiddle,
  wrapText,
  createSeparator,
} from './utils/formatCommand.js'

// Re-export spinner and output
export { createSpinner, withSpinner } from './spinner.js'
export {
  printCommand,
  printWarning,
  printError,
  printSuccess,
  printInfo,
} from './output.js'

// Re-export components
export { Spinner } from './components/Spinner.js'
export { CommandDisplay } from './components/CommandDisplay.js'
export { DangerousWarning } from './components/DangerousWarning.js'
export { ActionPrompt } from './components/ActionPrompt.js'
export { App } from './App.js'

/**
 * Render the interactive UI for command selection
 *
 * In TTY mode: Shows Ink-based interactive UI with keyboard navigation
 * In piped mode: Returns first command immediately without UI
 *
 * @param options - Render options with commands, config, and danger status
 * @returns Promise resolving to user action and selected command
 */
export function renderUI(options: RenderOptions): Promise<RenderResult> {
  const { commands, config, isDangerous } = options

  // Debug logging
  if (config.debug) {
    console.error('[UI] renderUI called')
    console.error(`[UI] Commands: ${commands.length}, Dangerous: ${isDangerous}`)
    console.error(
      `[UI] stdin.isTTY: ${process.stdin.isTTY}, stdout.isTTY: ${process.stdout.isTTY}`
    )
  }

  // Check if we're in TTY mode (interactive terminal)
  const isTTY = process.stdin.isTTY === true && process.stdout.isTTY === true

  // If not TTY (piped), return first command immediately without UI
  if (!isTTY) {
    if (config.debug) {
      console.error('[UI] Non-TTY mode, returning first command')
    }
    return Promise.resolve({
      action: UserAction.Execute,
      command: commands[0] ?? '',
    })
  }

  // TTY mode: render Ink UI
  if (config.debug) {
    console.error('[UI] TTY mode, rendering Ink UI')
  }

  return new Promise((resolve) => {
    const { unmount, waitUntilExit } = render(
      <App
        commands={commands}
        isDangerous={isDangerous}
        config={config}
        onComplete={(action, command) => {
          if (config.debug) {
            console.error(`[UI] onComplete: ${action}, ${command}`)
          }
          unmount()
          resolve({ action, command })
        }}
      />,
      {
        // Render to stderr so stdout stays clean for command output
        stdout: process.stderr,
        // Pass debug mode
        debug: config.debug,
      }
    )

    // Handle any errors during rendering
    waitUntilExit().catch((err) => {
      if (config.debug) {
        console.error('[UI] Render error:', err)
      }
      resolve({
        action: UserAction.Abort,
        command: commands[0] ?? '',
      })
    })
  })
}

export default renderUI
