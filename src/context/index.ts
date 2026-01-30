// Context gathering module - main entry point
// Gathers system info, directory context, shell history, and stdin

import { Config } from '../config/types.js'
import { ContextData, ContextError } from './types.js'
import { getSystemInfo } from './system.js'
import { getCwd, scanDirectory } from './directory.js'
import { getShellHistory } from './history.js'
import { readStdin } from './stdin.js'
import { redactPath } from './redaction.js'

// Re-export types and functions
export { ContextError } from './types.js'
export type { ContextData, SystemInfo } from './types.js'
export { getSystemInfo, clearSystemCache } from './system.js'
export { getCwd, scanDirectory, getRedactedCwd } from './directory.js'
export { getShellHistory, getDetectedShell } from './history.js'
export { readStdin, hasPipedStdin } from './stdin.js'
export { redactPath, redactUsername, redactEnvVars } from './redaction.js'

/**
 * Gather all context information for the AI prompt
 *
 * Components:
 * - System info (cached): OS, shell, user, memory
 * - Current working directory (fatal if fails)
 * - Directory files (sorted, truncated, redacted)
 * - Shell history (last N commands)
 * - Stdin content (only if piped, max 10KB)
 *
 * @param config - Runtime configuration with context settings
 * @returns ContextData with all gathered information
 * @throws ContextError if CWD cannot be determined (fatal)
 */
export async function gatherContext(config: Config): Promise<ContextData> {
  const { maxFiles, maxHistory, redactPaths, redactUsername } = config.context

  // 1. System info (cached, non-fatal)
  const system = getSystemInfo(redactUsername)

  // 2. CWD (fatal if fails)
  let cwd: string
  try {
    cwd = getCwd()
    if (redactPaths) {
      cwd = redactPath(cwd)
    }
  } catch (err) {
    if (err instanceof ContextError) {
      throw err
    }
    throw new ContextError(
      `Failed to gather context: ${err instanceof Error ? err.message : String(err)}`,
      1
    )
  }

  // 3. Directory files (non-fatal, empty on error)
  const files = scanDirectory(maxFiles, redactPaths)

  // 4. Shell history (non-fatal, empty on error)
  const history = getShellHistory(maxHistory)

  // 5. Stdin (only if piped, non-fatal)
  const stdin = await readStdin()

  return {
    system,
    cwd,
    files,
    history,
    stdin,
  }
}
