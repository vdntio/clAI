// src/safety/index.ts
// Safety module entry point

import type { Config } from '../config/types.js'
import type { CompiledPattern } from './types.js'
import { DEFAULT_DANGEROUS_PATTERNS, compilePatterns, isDangerous } from './patterns.js'

// Re-export types and functions
export { SafetyError } from './types.js'
export type { CompiledPattern } from './types.js'
export { DEFAULT_DANGEROUS_PATTERNS, compilePatterns, isDangerous } from './patterns.js'

/**
 * Load and compile dangerous patterns from config or defaults
 *
 * @param config - Application config
 * @returns Compiled patterns ready for matching
 */
export function loadPatterns(config: Config): CompiledPattern[] {
  const patterns =
    config.safety.dangerousPatterns.length > 0
      ? config.safety.dangerousPatterns
      : DEFAULT_DANGEROUS_PATTERNS

  return compilePatterns(patterns)
}

/**
 * Determine if we should show an interactive safety prompt
 * Returns true only when all conditions are met:
 * - Safety confirmation is enabled in config
 * - Force flag is not set
 * - Running in interactive TTY mode
 *
 * @param config - Application config
 * @returns true if safety prompt should be shown
 */
export function shouldPrompt(config: Config): boolean {
  // Don't prompt if safety confirmation is disabled
  if (!config.safety.confirmDangerous) {
    return false
  }

  // Don't prompt if force flag is set
  if (config.force) {
    return false
  }

  // Don't prompt in non-interactive mode (piped)
  if (process.stdin.isTTY !== true || process.stdout.isTTY !== true) {
    return false
  }

  return true
}

/**
 * Check commands for dangerous patterns and determine UI behavior
 *
 * @param commands - Commands to check
 * @param config - Application config
 * @returns Object with danger status and prompt requirement
 */
export function checkSafety(
  commands: string[],
  config: Config
): { isDangerous: boolean; shouldPrompt: boolean } {
  const patterns = loadPatterns(config)

  // Check if any command is dangerous
  const dangerous = commands.some((cmd) => isDangerous(cmd, patterns))

  return {
    isDangerous: dangerous,
    shouldPrompt: dangerous && shouldPrompt(config),
  }
}
