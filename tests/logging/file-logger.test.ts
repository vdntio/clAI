import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { FileLogger } from '../../src/logging/file-logger.js'
import * as fs from 'fs'
import * as path from 'path'

describe.sequential('FileLogger JSONL format', () => {
  const tmpDir = '/tmp/clai-test-logs-file-logger-jsonl'
  const tmpFile = path.join(tmpDir, 'test.log')

  beforeEach(() => {
    // Ensure directory exists
    try {
      fs.mkdirSync(tmpDir, { recursive: true })
    } catch (error) {
      // Directory may already exist
    }
  })

  afterEach(() => {
    // Clean up test files
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    } catch (error) {
      // Directory doesn't exist
    }
  })

  it('writes valid JSONL entries', () => {
    const logger = new FileLogger(tmpFile)
    logger.error('test message')

    const content = fs.readFileSync(tmpFile, 'utf-8')
    const lines = content.trim().split('\n')
    const entry = JSON.parse(lines[0] ?? '{}')

    expect(entry).toHaveProperty('ts')
    expect(entry).toHaveProperty('level', 'error')
    expect(entry).toHaveProperty('msg', 'test message')
    expect(new Date(entry.ts).toISOString()).toBe(entry.ts) // Valid ISO date
  })

  it('appends multiple entries as separate lines', () => {
    const logger = new FileLogger(tmpFile)
    logger.error('error 1')
    logger.warn('warning 1')
    logger.info('info 1')
    logger.debug('debug 1')

    const content = fs.readFileSync(tmpFile, 'utf-8')
    const lines = content.trim().split('\n')

    expect(lines).toHaveLength(4)

    const entry1 = JSON.parse(lines[0] ?? '{}')
    const entry2 = JSON.parse(lines[1] ?? '{}')
    const entry3 = JSON.parse(lines[2] ?? '{}')
    const entry4 = JSON.parse(lines[3] ?? '{}')

    expect(entry1.level).toBe('error')
    expect(entry1.msg).toBe('error 1')
    expect(entry2.level).toBe('warn')
    expect(entry2.msg).toBe('warning 1')
    expect(entry3.level).toBe('info')
    expect(entry3.msg).toBe('info 1')
    expect(entry4.level).toBe('debug')
    expect(entry4.msg).toBe('debug 1')
  })

  it('creates parent directory if it does not exist', () => {
    const nestedPath = path.join(tmpDir, 'nested', 'dir', 'test.log')
    const logger = new FileLogger(nestedPath)
    logger.error('test')

    expect(fs.existsSync(nestedPath)).toBe(true)
  })
})

describe.sequential('FileLogger size management', () => {
  const tmpDir = '/tmp/clai-test-logs-file-logger-size'
  const tmpFile = path.join(tmpDir, 'test.log')

  beforeEach(() => {
    // Ensure directory exists
    try {
      fs.mkdirSync(tmpDir, { recursive: true })
    } catch (error) {
      // Directory may already exist
    }
  })

  afterEach(() => {
    // Clean up test files
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    } catch (error) {
      // Directory doesn't exist
    }
  })

  it('truncates file when >10MB', () => {
    // Create a file >10MB
    fs.mkdirSync(tmpDir, { recursive: true })
    const largeContent = 'x'.repeat(11 * 1024 * 1024) // 11MB
    fs.writeFileSync(tmpFile, largeContent)

    const logger = new FileLogger(tmpFile)
    logger.error('test')

    const stats = fs.statSync(tmpFile)
    // File should be much smaller than 11MB now (just the new entry)
    expect(stats.size).toBeLessThan(1000)
  })

  it('does not truncate file when <10MB', () => {
    // Create a file <10MB
    fs.mkdirSync(tmpDir, { recursive: true })
    const smallContent = 'x'.repeat(100) // 100 bytes
    fs.writeFileSync(tmpFile, smallContent)

    const logger = new FileLogger(tmpFile)
    logger.error('test')

    const content = fs.readFileSync(tmpFile, 'utf-8')
    // File should contain both old content and new entry
    expect(content).toContain('x'.repeat(100))
    expect(content).toContain('test')
  })
})

describe.sequential('FileLogger error handling', () => {
  it('does not crash when writing to invalid path', () => {
    // Should not throw even with invalid path like /dev/null/invalid
    expect(() => {
      const logger = new FileLogger('/dev/null/invalid/path.log')
      logger.error('test')
    }).not.toThrow()
  })

  it('creates logger instance even if directory creation might fail', () => {
    // Should not throw during instantiation
    expect(() => {
      new FileLogger('/invalid/path/test.log')
    }).not.toThrow()
  })
})

describe.sequential('FileLogger does not use colors', () => {
  const tmpDir = '/tmp/clai-test-logs-file-logger-colors'
  const tmpFile = path.join(tmpDir, 'test.log')

  beforeEach(() => {
    // Ensure directory exists
    try {
      fs.mkdirSync(tmpDir, { recursive: true })
    } catch (error) {
      // Directory may already exist
    }
  })

  afterEach(() => {
    // Clean up test files
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true })
    } catch (error) {
      // Directory doesn't exist
    }
  })

  it('writes plain text without ANSI codes', () => {
    const logger = new FileLogger(tmpFile)
    logger.error('test error')

    const content = fs.readFileSync(tmpFile, 'utf-8')
    // Should not contain ANSI escape codes
    expect(content).not.toMatch(/\x1b\[/)
  })
})
