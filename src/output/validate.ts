// src/output/validate.ts
// Command validation before execution

import { Errors, type ValidationResult } from './types.js'

/**
 * Detect recursive clai invocation (AI suggesting clai commands → infinite loop)
 *
 * Matches:
 * - "clai" at start of command
 * - "./clai" at start of command
 * - "/path/to/clai" at start of command
 * - "clai" after pipe: "echo | clai"
 * - "clai" after &&: "cd foo && clai"
 * - "clai" after ;: "echo hi; clai"
 *
 * Does NOT match (no false positives):
 * - "claimant" (clai is substring of word)
 * - "echo clai" (clai is argument, not command)
 * - "/usr/bin/claim" (different command)
 */
export function isRecursiveCall(command: string): boolean {
  // Pattern explanation:
  // (?:^|[|;&]\s*) - Start of string OR after pipe/&&/; with optional whitespace
  // (?:\.\/|\/[\w\/]*)? - Optional ./ prefix or absolute path
  // clai - The literal "clai"
  // (?:\s|$) - Followed by whitespace or end of string (word boundary)
  const claiPattern = /(?:^|[|;&]\s*)(?:\.\/|\/[\w/]*)?clai(?:\s|$)/
  return claiPattern.test(command)
}

/**
 * Validate command before execution
 * Returns validation result with typed error if invalid
 */
export function validateCommand(command: string): ValidationResult {
  const trimmed = command.trim()

  if (!trimmed) {
    return { valid: false, error: Errors.emptyCommand() }
  }

  if (isRecursiveCall(trimmed)) {
    return { valid: false, error: Errors.recursiveCall() }
  }

  return { valid: true }
}
