import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { Logger } from '../../src/logging/logger.js'

describe('Logger level filtering', () => {
  let stderrSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    stderrSpy = vi.spyOn(process.stderr, 'write').mockImplementation(() => true)
  })

  afterEach(() => {
    stderrSpy.mockRestore()
  })

  it('quiet mode suppresses all messages', () => {
    const logger = new Logger('quiet')
    logger.error('test')
    logger.warn('test')
    logger.info('test')
    logger.debug('test')
    expect(stderrSpy).not.toHaveBeenCalled()
  })

  it('normal mode shows error and warn', () => {
    const logger = new Logger('normal')
    logger.error('error')
    logger.warn('warn')
    logger.info('info')
    logger.debug('debug')
    expect(stderrSpy).toHaveBeenCalledTimes(2)
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('error'))
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('warn'))
  })

  it('verbose mode shows all messages', () => {
    const logger = new Logger('verbose')
    logger.error('e')
    logger.warn('w')
    logger.info('i')
    logger.debug('d')
    expect(stderrSpy).toHaveBeenCalledTimes(4)
  })
})

describe('Logger color handling', () => {
  let stderrSpy: ReturnType<typeof vi.spyOn>
  const originalNoColor = process.env.NO_COLOR
  const originalTerm = process.env.TERM

  beforeEach(() => {
    stderrSpy = vi.spyOn(process.stderr, 'write').mockImplementation(() => true)
  })

  afterEach(() => {
    stderrSpy.mockRestore()
    if (originalNoColor === undefined) {
      delete process.env.NO_COLOR
    } else {
      process.env.NO_COLOR = originalNoColor
    }
    if (originalTerm === undefined) {
      delete process.env.TERM
    } else {
      process.env.TERM = originalTerm
    }
  })

  it('disables color when NO_COLOR is set', () => {
    process.env.NO_COLOR = '1'
    const logger = new Logger('normal', 'auto')
    logger.error('test')
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('[ERROR] test'))
    // Should not contain ANSI escape codes
    const call = stderrSpy.mock.calls[0]?.[0] as string
    expect(call).not.toMatch(/\x1b\[/)
  })

  it('disables color with never mode', () => {
    const logger = new Logger('normal', 'never')
    logger.error('test')
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('[ERROR] test'))
    const call = stderrSpy.mock.calls[0]?.[0] as string
    expect(call).not.toMatch(/\x1b\[/)
  })

  it('enables color with always mode', () => {
    const logger = new Logger('normal', 'always')
    logger.error('test')
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('test'))
    // Should contain ANSI escape codes when always enabled
    const call = stderrSpy.mock.calls[0]?.[0] as string
    expect(call).toMatch(/\x1b\[/)
  })

  it('disables color when TERM is dumb', () => {
    delete process.env.NO_COLOR
    process.env.TERM = 'dumb'
    const logger = new Logger('normal', 'auto')
    logger.error('test')
    const call = stderrSpy.mock.calls[0]?.[0] as string
    expect(call).not.toMatch(/\x1b\[/)
  })
})

describe('Logger generic log method', () => {
  let stderrSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    stderrSpy = vi.spyOn(process.stderr, 'write').mockImplementation(() => true)
  })

  afterEach(() => {
    stderrSpy.mockRestore()
  })

  it('log method delegates to specific level methods', () => {
    const logger = new Logger('verbose')
    logger.log('error', 'error message')
    logger.log('warn', 'warn message')
    logger.log('info', 'info message')
    logger.log('debug', 'debug message')
    expect(stderrSpy).toHaveBeenCalledTimes(4)
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('error message'))
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('warn message'))
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('info message'))
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('debug message'))
  })
})

describe('Logger message formatting', () => {
  let stderrSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    stderrSpy = vi.spyOn(process.stderr, 'write').mockImplementation(() => true)
  })

  afterEach(() => {
    stderrSpy.mockRestore()
  })

  it('formats messages with level prefix', () => {
    const logger = new Logger('verbose', 'never')
    logger.error('test error')
    logger.warn('test warn')
    logger.info('test info')
    logger.debug('test debug')

    expect(stderrSpy).toHaveBeenCalledWith('[ERROR] test error\n')
    expect(stderrSpy).toHaveBeenCalledWith('[WARN] test warn\n')
    expect(stderrSpy).toHaveBeenCalledWith('[INFO] test info\n')
    expect(stderrSpy).toHaveBeenCalledWith('[DEBUG] test debug\n')
  })
})
