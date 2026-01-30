// src/output/execute.ts
// Command execution with proper error handling and exit code propagation

import { spawn, type ChildProcess } from 'child_process'
import { Errors, type ExecutionResult } from './types.js'
import { validateCommand } from './validate.js'

/**
 * Get the user's shell from environment, falling back to /bin/sh
 */
export function getShell(): string {
  return process.env.SHELL || '/bin/sh'
}

/**
 * Options for command execution
 */
export interface ExecuteOptions {
  /** Shell to use (defaults to user's SHELL or /bin/sh) */
  shell?: string
  /** Timeout in milliseconds (0 = no timeout) */
  timeout?: number
  /** Whether to inherit stdio (default: true for interactive) */
  inheritStdio?: boolean
}

/**
 * Execute a shell command and return the result
 *
 * Handles various edge cases:
 * - Empty command → error with code 1
 * - Recursive clai call → error with code 5 (safety)
 * - Shell not found → error with code 127
 * - Command not found → error with code 127
 * - Permission denied → error with code 126
 * - Signal termination → error with code 128
 * - Timeout → error with code 124
 * - Spawn failure → error with code 1
 */
export function executeCommand(
  command: string,
  options: ExecuteOptions = {}
): Promise<ExecutionResult> {
  return new Promise((resolve) => {
    // Validate command first
    const validation = validateCommand(command)
    if (!validation.valid) {
      resolve({ success: false, error: validation.error })
      return
    }

    const shell = options.shell ?? getShell()
    const inheritStdio = options.inheritStdio ?? true

    let child: ChildProcess
    let timeoutId: ReturnType<typeof setTimeout> | undefined
    let killed = false

    try {
      child = spawn(shell, ['-c', command], {
        stdio: inheritStdio ? 'inherit' : 'pipe',
        env: process.env,
      })
    } catch (err) {
      // Spawn itself threw (rare, but possible)
      resolve({
        success: false,
        error: Errors.spawnFailed(command, err instanceof Error ? err : new Error(String(err))),
      })
      return
    }

    // Set up timeout if specified
    if (options.timeout && options.timeout > 0) {
      timeoutId = setTimeout(() => {
        killed = true
        child.kill('SIGTERM')
        // Give it a moment, then SIGKILL if still alive
        setTimeout(() => {
          if (!child.killed) {
            child.kill('SIGKILL')
          }
        }, 1000)
      }, options.timeout)
    }

    child.on('close', (code, signal) => {
      if (timeoutId) {
        clearTimeout(timeoutId)
      }

      // Handle timeout
      if (killed) {
        resolve({
          success: false,
          error: Errors.timeout(options.timeout!),
        })
        return
      }

      // Handle signal termination
      if (signal) {
        resolve({
          success: false,
          error: Errors.signalKilled(signal),
        })
        return
      }

      // Handle exit codes
      const exitCode = code ?? 0

      // Check for special exit codes that indicate errors
      if (exitCode === 127) {
        // Could be shell not found or command not found
        // We can't easily distinguish, so use generic message
        resolve({
          success: false,
          error: Errors.commandNotFound(command.split(/\s+/)[0] || command),
        })
        return
      }

      if (exitCode === 126) {
        resolve({
          success: false,
          error: Errors.permissionDenied(command.split(/\s+/)[0] || command),
        })
        return
      }

      // All other exit codes: return as success with the exit code
      // (The caller decides if non-zero is an error for their use case)
      resolve({ success: true, exitCode })
    })

    child.on('error', (err) => {
      if (timeoutId) {
        clearTimeout(timeoutId)
      }

      // Check for specific error types
      const errMsg = err.message.toLowerCase()

      if (errMsg.includes('enoent') || errMsg.includes('not found')) {
        // Shell or command not found
        if (errMsg.includes(shell)) {
          resolve({
            success: false,
            error: Errors.shellNotFound(shell),
          })
        } else {
          resolve({
            success: false,
            error: Errors.commandNotFound(command.split(/\s+/)[0] || command),
          })
        }
        return
      }

      if (errMsg.includes('eacces') || errMsg.includes('permission')) {
        resolve({
          success: false,
          error: Errors.permissionDenied(command.split(/\s+/)[0] || command),
        })
        return
      }

      // Generic spawn error
      resolve({
        success: false,
        error: Errors.spawnFailed(command, err),
      })
    })
  })
}
