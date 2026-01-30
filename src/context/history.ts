// Shell history gathering

import { readFileSync } from 'fs'
import { join } from 'path'
import { homedir } from 'os'

// History file paths by shell
const HISTORY_PATHS: Record<string, string[]> = {
  bash: ['.bash_history'],
  zsh: ['.zsh_history'],
  fish: ['.local/share/fish/fish_history'],
  sh: ['.sh_history'],
}

/**
 * Detect shell from SHELL environment variable
 * Returns shell name (e.g., 'bash', 'zsh') or null if unknown
 */
function detectShell(): string | null {
  const shell = process.env.SHELL || ''
  if (!shell) return null

  const shellName = shell.split('/').pop() || ''

  // Check if we support this shell
  if (HISTORY_PATHS[shellName]) {
    return shellName
  }

  return null
}

/**
 * Get history file path for a given shell
 */
function getHistoryPath(shell: string): string | null {
  const home = homedir()
  if (!home) return null

  const paths = HISTORY_PATHS[shell]
  if (!paths) return null

  return join(home, paths[0]!)
}

/**
 * Read last N lines from a file efficiently
 * Uses tail-like approach: seek to end minus ~4KB, read, take last N lines
 */
function readLastLines(filePath: string, numLines: number): string[] {
  try {
    // Read entire file - for most history files this is fine
    // For very large files, we'd use a streaming approach
    const content = readFileSync(filePath, 'utf-8')
    const lines = content.split('\n')

    // Filter out empty lines and get last N
    const nonEmptyLines = lines.filter((line) => line.trim().length > 0)
    return nonEmptyLines.slice(-numLines)
  } catch {
    // File doesn't exist or can't be read
    return []
  }
}

/**
 * Get shell history
 * Returns last N commands based on detected shell
 *
 * - bash: ~/.bash_history
 * - zsh: ~/.zsh_history
 * - fish: ~/.local/share/fish/fish_history (read raw lines)
 * - other: empty array
 *
 * On error or unsupported shell, returns empty array (non-fatal)
 */
export function getShellHistory(maxHistory: number): string[] {
  const shell = detectShell()
  if (!shell) {
    return []
  }

  const historyPath = getHistoryPath(shell)
  if (!historyPath) {
    return []
  }

  const lines = readLastLines(historyPath, maxHistory)

  // For fish shell, we read raw lines (matching Rust behavior)
  // Fish uses format: "- cmd: <command>" but we return raw lines
  // This matches the PRD: "The Rust code does NOT parse fish's - cmd: ... format"

  return lines
}

/**
 * Get detected shell name (for testing/debugging)
 */
export function getDetectedShell(): string | null {
  return detectShell()
}
