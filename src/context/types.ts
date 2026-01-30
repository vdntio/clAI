// Context Data Types for the clai CLI
import { ClaiError } from '../error/index.js'

// System Information Interface
export interface SystemInfo {
  /** Operating system name (e.g., 'Linux', 'Darwin', 'Windows_NT') */
  osName: string
  /** OS version string */
  osVersion: string
  /** CPU architecture (e.g., 'x64', 'arm64') */
  architecture: string
  /** Shell name (e.g., 'bash', 'zsh', 'fish') */
  shell: string
  /** Username (may be redacted) */
  user: string
  /** Total system memory in MB */
  totalMemoryMb: number
}

export interface ContextData {
  /** System information */
  system: SystemInfo
  /** Current working directory (may be redacted) */
  cwd: string
  /** List of files in current directory (sorted, truncated, redacted) */
  files: string[]
  /** Recent shell history entries */
  history: string[]
  /** Piped stdin content (only when stdin is not a TTY) */
  stdin?: string
}

/** Error class for context gathering failures */
export class ContextError extends ClaiError {
  constructor(message: string, code: number = 1) {
    super(message, code)
    this.name = 'ContextError'
    Object.setPrototypeOf(this, ContextError.prototype)
  }
}
