import { Logger } from './logger.js'
import type { LogLevel } from './logger.js'
import { FileLogger } from './file-logger.js'

export { Logger, FileLogger }
export type { LogLevel }

// Multi-target logger that writes to both stderr and file
export class CombinedLogger {
  private stderrLogger: Logger
  private fileLogger?: FileLogger

  constructor(
    level: LogLevel,
    colorMode: 'auto' | 'always' | 'never',
    filePath?: string
  ) {
    this.stderrLogger = new Logger(level, colorMode)
    if (filePath) {
      this.fileLogger = new FileLogger(filePath, level)
    }
  }

  error(msg: string): void {
    this.stderrLogger.error(msg)
    this.fileLogger?.error(msg)
  }

  warn(msg: string): void {
    this.stderrLogger.warn(msg)
    this.fileLogger?.warn(msg)
  }

  info(msg: string): void {
    this.stderrLogger.info(msg)
    this.fileLogger?.info(msg)
  }

  debug(msg: string): void {
    this.stderrLogger.debug(msg)
    this.fileLogger?.debug(msg)
  }

  log(level: 'error' | 'warn' | 'info' | 'debug', msg: string): void {
    this[level](msg)
  }
}
