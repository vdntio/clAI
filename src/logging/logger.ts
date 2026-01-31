import chalk, { Chalk } from 'chalk'

export type LogLevel = 'quiet' | 'normal' | 'verbose'

export class Logger {
  private level: LogLevel
  private colorEnabled: boolean
  private chalk: InstanceType<typeof Chalk>

  constructor(level: LogLevel = 'normal', colorMode: 'auto' | 'always' | 'never' = 'auto') {
    this.level = level
    this.colorEnabled = this.resolveColorMode(colorMode)
    // Create chalk instance with explicit level
    this.chalk = this.colorEnabled ? new Chalk({ level: 3 }) : new Chalk({ level: 0 })
  }

  private resolveColorMode(mode: 'auto' | 'always' | 'never'): boolean {
    if (mode === 'never' || process.env.NO_COLOR) return false
    if (mode === 'always') return true
    // auto: use TTY + TERM check
    return process.stderr.isTTY && process.env.TERM !== 'dumb'
  }

  protected shouldLog(messageLevel: LogLevel): boolean {
    const levels = { quiet: 0, normal: 1, verbose: 2 }
    return levels[messageLevel] <= levels[this.level]
  }

  private formatMessage(level: string, msg: string): string {
    if (!this.colorEnabled) return `[${level.toUpperCase()}] ${msg}`

    const prefixMap: Record<string, string> = {
      error: this.chalk.red('[ERROR]'),
      warn: this.chalk.yellow('[WARN]'),
      info: this.chalk.cyan('[INFO]'),
      debug: this.chalk.dim('[DEBUG]'),
    }

    const prefix = prefixMap[level] || `[${level.toUpperCase()}]`
    return `${prefix} ${msg}`
  }

  error(msg: string): void {
    if (this.shouldLog('normal')) {
      process.stderr.write(this.formatMessage('error', msg) + '\n')
    }
  }

  warn(msg: string): void {
    if (this.shouldLog('normal')) {
      process.stderr.write(this.formatMessage('warn', msg) + '\n')
    }
  }

  info(msg: string): void {
    if (this.shouldLog('verbose')) {
      process.stderr.write(this.formatMessage('info', msg) + '\n')
    }
  }

  debug(msg: string): void {
    if (this.shouldLog('verbose')) {
      process.stderr.write(this.formatMessage('debug', msg) + '\n')
    }
  }

  // Generic log method
  log(level: 'error' | 'warn' | 'info' | 'debug', msg: string): void {
    this[level](msg)
  }
}
