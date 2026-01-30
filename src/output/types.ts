// src/output/types.ts
// Types for command execution and result handling
import { ClaiError } from '../error/index.js'

/**
 * Execution result type (functional Result pattern)
 * Either success with exit code, or failure with typed error
 */
export type ExecutionResult =
  | { success: true; exitCode: number }
  | { success: false; error: ExecutionError }

/**
 * Validation result for command validation
 */
export type ValidationResult =
  | { valid: true }
  | { valid: false; error: ExecutionError }

/**
 * ExecutionError represents failures during command execution.
 * Each error type has a specific exit code following shell conventions.
 */
export class ExecutionError extends ClaiError {
  constructor(
    message: string,
    code: number,
    cause?: Error
  ) {
    super(message, code, cause)
    this.name = 'ExecutionError'
    Object.setPrototypeOf(this, ExecutionError.prototype)
  }
}

/**
 * Error factory functions (pure, testable)
 * Exit codes follow shell conventions:
 * - 1: General error
 * - 5: Safety abort (clai-specific)
 * - 124: Timeout (following GNU coreutils timeout)
 * - 126: Permission denied (POSIX)
 * - 127: Command not found (POSIX)
 * - 128+N: Killed by signal N
 */
export const Errors = {
  emptyCommand: () =>
    new ExecutionError('Empty command', 1),

  spawnFailed: (cmd: string, err: Error) =>
    new ExecutionError(`Failed to spawn: ${cmd}`, 1, err),

  shellNotFound: (shell: string) =>
    new ExecutionError(`Shell not found: ${shell}`, 127),

  commandNotFound: (cmd: string) =>
    new ExecutionError(`Command not found: ${cmd}`, 127),

  permissionDenied: (cmd: string) =>
    new ExecutionError(`Permission denied: ${cmd}`, 126),

  signalKilled: (signal: string) =>
    new ExecutionError(`Killed by signal: ${signal}`, 128),

  timeout: (timeoutMs: number) =>
    new ExecutionError(`Command timed out after ${timeoutMs}ms`, 124),

  recursiveCall: () =>
    new ExecutionError(
      'Refusing to execute clai recursively (would cause infinite loop)',
      5
    ),
} as const
