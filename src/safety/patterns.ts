// src/safety/patterns.ts
// Dangerous command pattern detection

import type { CompiledPattern } from './types.js'

/**
 * Default dangerous patterns - regex strings that match potentially destructive commands
 * These are compiled to RegExp at runtime for matching
 */
export const DEFAULT_DANGEROUS_PATTERNS: readonly string[] = [
  // File deletion patterns
  'rm\\s+(-[^\\s]*)?\\s*-rf\\s+/',        // rm -rf / (root wipe)
  'rm\\s+(-[^\\s]*)?\\s*-f',               // rm with force flag
  'rm\\s+(-[^\\s]*r[^\\s]*|[^\\s]*r)\\s+', // rm with recursive
  'find\\s+.*-exec\\s+(rm|del)',           // find ... -exec rm/del

  // Disk/device operations
  'dd\\s+.*if=/dev/(zero|random|urandom)', // disk wipe with dd
  'dd\\s+.*of=/dev/',                       // write to raw device
  'mkfs\\.\\w+\\s+/dev/',                   // format filesystem
  'mkfs\\s+-t\\s+\\w+\\s+/dev/',            // mkfs with type flag
  'shred\\s+',                              // secure delete

  // Privileged destruction
  'sudo\\s+rm\\s+(-[^\\s]*)?\\s*-rf',       // sudo rm -rf
  'sudo\\s+dd\\s+',                         // sudo dd
  'sudo\\s+mkfs',                           // sudo mkfs

  // Redirects to devices
  '>\\s*/dev/sd[a-z]',                      // redirect to disk device
  '>\\s*/dev/nvme',                         // redirect to nvme
  '>\\s*/dev/null.*<',                      // suspicious null redirect

  // Database destruction
  'drop\\s+database',                       // SQL drop database
  'drop\\s+table',                          // SQL drop table
  'truncate\\s+table',                      // SQL truncate
  'delete\\s+from\\s+\\w+\\s*;?$',          // DELETE without WHERE

  // Git destructive operations
  'git\\s+reset\\s+--hard',                 // discard all changes
  'git\\s+clean\\s+-fd',                    // remove untracked files
  'git\\s+push\\s+.*--force',               // force push

  // System modification
  'chmod\\s+(-R\\s+)?777\\s+/',             // world-writable root
  'chown\\s+-R\\s+.*:\\s*/',                // recursive chown on root
  ':(){ :|:& };:',                          // fork bomb

  // Windows-specific (for cross-platform awareness)
  'format\\s+[a-z]:',                       // format drive
  'del\\s+/[fqs]',                          // del with force/quiet/subdirs
  'rd\\s+/s\\s+/q',                         // rmdir /s /q
] as const

/**
 * Compile pattern strings into RegExp objects
 * Invalid patterns are marked with isValid: false
 *
 * @param patterns - Array of regex pattern strings
 * @returns Array of compiled patterns with validity status
 */
export function compilePatterns(patterns: readonly string[]): CompiledPattern[] {
  return patterns.map((pattern) => {
    try {
      return {
        pattern,
        regex: new RegExp(pattern, 'i'),
        isValid: true,
      }
    } catch {
      // Invalid regex - mark as invalid (will trigger fail-safe)
      return {
        pattern,
        regex: null,
        isValid: false,
      }
    }
  })
}

/**
 * Check if a command matches any dangerous pattern
 * Fail-safe: if any pattern is invalid, treat all commands as dangerous
 *
 * @param command - The command string to check
 * @param compiledPatterns - Array of compiled patterns
 * @returns true if command is dangerous, false if safe
 */
export function isDangerous(
  command: string,
  compiledPatterns: CompiledPattern[]
): boolean {
  // Empty command is safe
  if (!command.trim()) {
    return false
  }

  // Fail-safe: if any pattern failed to compile, treat as dangerous
  const hasInvalidPattern = compiledPatterns.some((p) => !p.isValid)
  if (hasInvalidPattern) {
    return true
  }

  // Check each valid pattern for a match
  for (const { regex } of compiledPatterns) {
    if (regex && regex.test(command)) {
      return true
    }
  }

  return false
}
