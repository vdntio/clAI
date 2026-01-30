// Directory context gathering

import { readdirSync } from 'fs'
import { basename } from 'path'
import { ContextError } from './types.js'
import { redactPath } from './redaction.js'

const PATH_TRUNCATE_LENGTH = 80

/**
 * Get current working directory
 * Throws ContextError if cwd cannot be determined (fatal)
 */
export function getCwd(): string {
  try {
    return process.cwd()
  } catch (err) {
    throw new ContextError(
      `Failed to get current working directory: ${err instanceof Error ? err.message : String(err)}`,
      1
    )
  }
}

/**
 * Truncate a path to maximum length
 * If path exceeds limit, return basename only
 */
function truncatePath(
  path: string,
  maxLength: number = PATH_TRUNCATE_LENGTH
): string {
  if (path.length <= maxLength) {
    return path
  }
  return basename(path)
}

/**
 * Scan current directory and return list of files
 * - Limited to maxFiles (sorted alphabetically)
 * - Paths truncated to 80 chars (using basename if too long)
 * - Optionally redacted for privacy
 *
 * On read error, returns empty array (non-fatal)
 */
export function scanDirectory(
  maxFiles: number,
  redactPaths: boolean
): string[] {
  const cwd = getCwd()

  try {
    // Read directory entries
    const entries = readdirSync(cwd, { withFileTypes: true })

    // Sort by name and take first maxFiles
    const sorted = entries
      .map((entry) => entry.name)
      .sort((a, b) => a.localeCompare(b))
      .slice(0, maxFiles)

    // Apply truncation and redaction
    return sorted.map((fileName) => {
      const fullPath = `${cwd}/${fileName}`
      let result = truncatePath(fullPath)

      if (redactPaths) {
        result = redactPath(result)
      }

      return result
    })
  } catch {
    // Non-fatal: return empty list on error
    return []
  }
}

/**
 * Get redacted current working directory
 */
export function getRedactedCwd(redactPaths: boolean): string {
  const cwd = getCwd()
  if (redactPaths) {
    return redactPath(cwd)
  }
  return cwd
}
