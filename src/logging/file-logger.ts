import { appendFileSync, statSync, truncateSync, mkdirSync } from 'fs'
import { dirname } from 'path'
import { Logger, LogLevel } from './logger.js'

const MAX_FILE_SIZE = 10 * 1024 * 1024 // 10MB

export class FileLogger extends Logger {
  private filePath: string

  constructor(filePath: string, level: LogLevel = 'verbose') {
    super(level, 'never') // File logging never uses color
    this.filePath = filePath
    this.ensureDirectory()
  }

  private ensureDirectory(): void {
    try {
      mkdirSync(dirname(this.filePath), { recursive: true })
    } catch (error) {
      // Directory may already exist
    }
  }

  private checkAndTruncate(): void {
    try {
      const stats = statSync(this.filePath)
      if (stats.size > MAX_FILE_SIZE) {
        truncateSync(this.filePath, 0)
      }
    } catch (error) {
      // File doesn't exist yet, will be created on first write
    }
  }

  private writeToFile(level: string, msg: string): void {
    this.checkAndTruncate()

    const entry = {
      ts: new Date().toISOString(),
      level,
      msg,
    }

    const line = JSON.stringify(entry) + '\n'

    try {
      appendFileSync(this.filePath, line, 'utf-8')
    } catch (error) {
      // Silent fail - don't crash app if file logging fails
    }
  }

  override error(msg: string): void {
    if (this.shouldLog('normal')) {
      this.writeToFile('error', msg)
    }
  }

  override warn(msg: string): void {
    if (this.shouldLog('normal')) {
      this.writeToFile('warn', msg)
    }
  }

  override info(msg: string): void {
    if (this.shouldLog('verbose')) {
      this.writeToFile('info', msg)
    }
  }

  override debug(msg: string): void {
    if (this.shouldLog('verbose')) {
      this.writeToFile('debug', msg)
    }
  }
}
