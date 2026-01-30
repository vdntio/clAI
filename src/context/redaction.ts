// Path and username redaction helpers for privacy

import { homedir } from 'os'

const REDACTED = '[REDACTED]'

/**
 * Redact sensitive path information from a string
 * Replaces home directory patterns with [REDACTED]
 */
export function redactPath(path: string): string {
  const home = homedir()
  if (!home) return path

  // Normalize path separators for Windows
  const normalizedPath = path.replace(/\\/g, '/')
  const normalizedHome = home.replace(/\\/g, '/')

  // Extract username from home path (e.g., /home/username or /Users/username)
  const homeParts = normalizedHome.split('/')
  const username = homeParts[homeParts.length - 1] || ''

  let redacted = normalizedPath

  // Replace full home directory path
  if (normalizedHome) {
    const escapedHome = normalizedHome.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    redacted = redacted.replace(new RegExp(escapedHome, 'g'), REDACTED)
  }

  // Replace ~ (home shorthand)
  redacted = redacted.replace(/^~\//, `${REDACTED}/`)
  redacted = redacted.replace(/^~$/, REDACTED)

  // Replace /home/username/ pattern
  if (username) {
    redacted = redacted.replace(
      new RegExp(`/home/${username}/`, 'g'),
      `${REDACTED}/`
    )
    redacted = redacted.replace(
      new RegExp(`/Users/${username}/`, 'g'),
      `${REDACTED}/`
    )
  }

  // Restore original path separators for Windows
  if (process.platform === 'win32') {
    redacted = redacted.replace(/\//g, '\\')
  }

  return redacted
}

/**
 * Redact username
 */
export function redactUsername(_username: string): string {
  return REDACTED
}

/**
 * Redact environment variables in a string
 * Replaces $VAR and ${VAR} patterns
 */
export function redactEnvVars(text: string, envVars: string[]): string {
  let redacted = text
  for (const envVar of envVars) {
    // Replace ${VAR} pattern
    redacted = redacted.replace(new RegExp(`\\$\\{${envVar}\\}`, 'g'), REDACTED)
    // Replace $VAR pattern (word boundary)
    redacted = redacted.replace(new RegExp(`\\$${envVar}\\b`, 'g'), REDACTED)
  }
  return redacted
}
