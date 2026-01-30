// System information gathering - cached per process

import { type, release, totalmem } from 'os'
import { SystemInfo } from './types.js'
import { redactUsername } from './redaction.js'

// Cached system info to avoid repeated OS calls
let cachedSystemInfo: SystemInfo | null = null
let cachedWithRedaction = false

/**
 * Get shell name from SHELL environment variable
 * Returns basename (e.g., '/bin/bash' -> 'bash')
 */
function getShell(): string {
  const shell = process.env.SHELL || ''
  if (!shell) return 'unknown'

  // Extract basename from path
  const parts = shell.split('/')
  return parts[parts.length - 1] || 'unknown'
}

/**
 * Get username from environment variables
 */
function getUsername(): string {
  return process.env.USER || process.env.USERNAME || 'unknown'
}

/**
 * Get system information
 * Results are cached for the process lifetime
 * @param redactUser - Whether to redact the username
 */
export function getSystemInfo(redactUser: boolean = false): SystemInfo {
  // Return cached value if available and redaction matches
  if (cachedSystemInfo && cachedWithRedaction === redactUser) {
    return cachedSystemInfo
  }

  const rawUsername = getUsername()
  const user = redactUser ? redactUsername(rawUsername) : rawUsername

  const info: SystemInfo = {
    osName: type(),
    osVersion: release(),
    architecture: process.arch,
    shell: getShell(),
    user,
    totalMemoryMb: Math.floor(totalmem() / (1024 * 1024)),
  }

  // Cache the result
  cachedSystemInfo = info
  cachedWithRedaction = redactUser

  return info
}

/**
 * Clear the system info cache (useful for testing)
 */
export function clearSystemCache(): void {
  cachedSystemInfo = null
  cachedWithRedaction = false
}
