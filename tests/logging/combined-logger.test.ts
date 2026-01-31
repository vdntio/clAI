import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { CombinedLogger } from '../../src/logging/index.js'
import * as fs from 'fs'
import * as path from 'path'

describe.sequential('CombinedLogger', () => {
  const tmpDir = '/tmp/clai-test-logs-combined-logger'
  let stderrSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    stderrSpy = vi.spyOn(process.stderr, 'write').mockImplementation(() => true)
    // Ensure directory exists
    try {
      fs.mkdirSync(tmpDir, { recursive: true })
    } catch (error) {
      // Directory may already exist
    }
  })

  afterEach(() => {
    stderrSpy.mockRestore()
    // Clean up test files
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    } catch (error) {
      // Directory doesn't exist
    }
  })

  it('writes to both stderr and file when file path is provided', () => {
    const tmpFile = path.join(tmpDir, 'test-1.log')
    const logger = new CombinedLogger('verbose', 'never', tmpFile)
    logger.error('test error')

    // Check stderr
    expect(stderrSpy).toHaveBeenCalledWith('[ERROR] test error\n')

    // Check file
    const content = fs.readFileSync(tmpFile, 'utf-8')
    const entry = JSON.parse(content.trim())
    expect(entry.level).toBe('error')
    expect(entry.msg).toBe('test error')
  })

  it('only writes to stderr when file path is not provided', () => {
    const tmpFile = path.join(tmpDir, 'test-2.log')
    const logger = new CombinedLogger('verbose', 'never')
    logger.error('test error')

    // Check stderr
    expect(stderrSpy).toHaveBeenCalledWith('[ERROR] test error\n')

    // File should not exist
    expect(fs.existsSync(tmpFile)).toBe(false)
  })

  it('respects log level for both targets', () => {
    const tmpFile = path.join(tmpDir, 'test-3.log')
    const logger = new CombinedLogger('normal', 'never', tmpFile)
    logger.error('error')
    logger.warn('warn')
    logger.info('info')
    logger.debug('debug')

    // Check stderr (should only have error and warn)
    expect(stderrSpy).toHaveBeenCalledTimes(2)

    // Check file (should only have error and warn)
    const content = fs.readFileSync(tmpFile, 'utf-8')
    const lines = content.trim().split('\n')
    expect(lines).toHaveLength(2)

    const entry1 = JSON.parse(lines[0] ?? '{}')
    const entry2 = JSON.parse(lines[1] ?? '{}')
    expect(entry1.level).toBe('error')
    expect(entry2.level).toBe('warn')
  })

  it('generic log method works for both targets', () => {
    const tmpFile = path.join(tmpDir, 'test-4.log')
    const logger = new CombinedLogger('verbose', 'never', tmpFile)
    logger.log('error', 'error message')
    logger.log('warn', 'warn message')

    // Check stderr
    expect(stderrSpy).toHaveBeenCalledTimes(2)
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('error message'))
    expect(stderrSpy).toHaveBeenCalledWith(expect.stringContaining('warn message'))

    // Check file
    expect(fs.existsSync(tmpFile)).toBe(true)
    const content = fs.readFileSync(tmpFile, 'utf-8')
    const lines = content.trim().split('\n')
    expect(lines).toHaveLength(2)
  })

  it('applies color mode to stderr but not file', () => {
    const tmpFile = path.join(tmpDir, 'test-5.log')
    const logger = new CombinedLogger('normal', 'always', tmpFile)
    logger.error('test error')

    // Check stderr (should have ANSI codes)
    const stderrCall = stderrSpy.mock.calls[0]?.[0] as string
    expect(stderrCall).toMatch(/\x1b\[/)

    // Check file (should NOT have ANSI codes)
    const content = fs.readFileSync(tmpFile, 'utf-8')
    expect(content).not.toMatch(/\x1b\[/)
  })
})
