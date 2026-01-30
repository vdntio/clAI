// src/safety/types.ts
// Safety error type for abort/timeout scenarios
import { ClaiError } from '../error/index.js'

/**
 * SafetyError is thrown when:
 * - User aborts a dangerous command
 * - Confirmation timeout expires
 * - Safety check fails
 *
 * Exit code: 5
 */
export class SafetyError extends ClaiError {
  constructor(message: string, cause?: Error) {
    super(message, 5, cause)
    this.name = 'SafetyError'
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
