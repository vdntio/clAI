// src/safety/types.ts
// Safety error type for abort/timeout scenarios

/**
 * SafetyError is thrown when:
 * - User aborts a dangerous command
 * - Confirmation timeout expires
 * - Safety check fails
 *
 * Exit code: 5
 */
export class SafetyError extends Error {
  readonly code = 5

  constructor(message: string) {
    super(message)
    this.name = 'SafetyError'
    // Maintain proper prototype chain for instanceof checks
    Object.setPrototypeOf(this, SafetyError.prototype)
  }
}

/**
 * Compiled pattern with validity status
 */
export interface CompiledPattern {
  pattern: string
  regex: RegExp | null
  isValid: boolean
}
