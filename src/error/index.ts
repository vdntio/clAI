/**
 * Base error class for all clai errors with exit code semantics
 *
 * Exit codes:
 * 0: Success (help/version)
 * 1: General/unhandled errors
 * 2: Usage errors (invalid CLI arguments)
 * 3: Config errors (parse failures, permissions)
 * 4: API errors (auth, rate limit, timeout)
 * 5: Safety errors (user abort)
 * 130: Interrupted (SIGINT/SIGTERM)
 */
export class ClaiError extends Error {
  public readonly code!: number

  constructor(message: string, code: number = 1, cause?: Error) {
    super(message, { cause })
    this.name = 'ClaiError'

    // Define code as non-writable property
    Object.defineProperty(this, 'code', {
      value: code,
      writable: false,
      enumerable: true,
      configurable: false,
    })

    // Maintain proper prototype chain for instanceof checks
    Object.setPrototypeOf(this, ClaiError.prototype)
  }
}

/**
 * Usage error: invalid CLI arguments or options
 * Exit code: 2
 */
export class UsageError extends ClaiError {
  constructor(message: string, cause?: Error) {
    super(message, 2, cause)
    this.name = 'UsageError'
    Object.setPrototypeOf(this, UsageError.prototype)
  }
}

/**
 * Interrupt error: SIGINT/SIGTERM received
 * Exit code: 130
 */
export class InterruptError extends ClaiError {
  constructor(message: string = 'Interrupted', cause?: Error) {
    super(message, 130, cause)
    this.name = 'InterruptError'
    Object.setPrototypeOf(this, InterruptError.prototype)
  }
}
