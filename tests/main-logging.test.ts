import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import { execSync } from 'child_process'
import * as fs from 'fs'
import * as path from 'path'

describe.sequential('Main CLI logging integration', () => {
  const tmpDir = '/tmp/clai-test-logs-main-logging'
  const tmpFile = path.join(tmpDir, 'integration-test.log')

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

  it('creates debug file with JSONL entries when --debug-file is provided', () => {
    const cmd = `bun run dist/main.js --debug --debug-file ${tmpFile} "echo test" 2>&1`
    try {
      execSync(cmd, { encoding: 'utf-8' })
    } catch (error) {
      // Command may exit with non-zero, but that's ok for this test
    }

    expect(fs.existsSync(tmpFile)).toBe(true)

    const content = fs.readFileSync(tmpFile, 'utf-8')
    const lines = content.trim().split('\n')

    // Should have multiple debug entries
    expect(lines.length).toBeGreaterThan(5)

    // Validate JSONL format
    lines.forEach((line) => {
      const entry = JSON.parse(line)
      expect(entry).toHaveProperty('ts')
      expect(entry).toHaveProperty('level')
      expect(entry).toHaveProperty('msg')
      expect(new Date(entry.ts).toISOString()).toBe(entry.ts)
    })
  })

  it('does not create debug file when --debug-file is not provided', () => {
    const differentFile = path.join(tmpDir, 'should-not-exist.log')
    const cmd = `bun run dist/main.js --debug "echo test" 2>&1`
    try {
      execSync(cmd, { encoding: 'utf-8' })
    } catch (error) {
      // Command may exit with non-zero, but that's ok for this test
    }

    expect(fs.existsSync(differentFile)).toBe(false)
  })

  it('quiet mode suppresses stderr output', () => {
    const cmd = `bun run dist/main.js -q "echo test" 2>&1`
    let output = ''
    try {
      output = execSync(cmd, { encoding: 'utf-8' })
    } catch (error) {
      // Command may exit with non-zero, but that's ok for this test
    }

    // Should have no output (or just the command on stdout)
    const lines = output.trim().split('\n').filter((line) => line.length > 0)
    expect(lines.length).toBeLessThanOrEqual(1)
  })

  it('debug mode shows debug messages to stderr', () => {
    const cmd = `bun run dist/main.js --debug "echo test" 2>&1`
    let output = ''
    try {
      output = execSync(cmd, { encoding: 'utf-8' })
    } catch (error) {
      // Command may exit with non-zero, but that's ok for this test
    }

    expect(output).toContain('[DEBUG]')
    expect(output).toContain('=== Loaded Config ===')
  })

  it('color never mode produces plain text without ANSI codes', () => {
    const cmd = `bun run dist/main.js --debug --color never "echo test" 2>&1`
    let output = ''
    try {
      output = execSync(cmd, { encoding: 'utf-8' })
    } catch (error) {
      // Command may exit with non-zero, but that's ok for this test
    }

    // Should not contain ANSI escape codes
    expect(output).not.toMatch(/\x1b\[/)
    expect(output).toContain('[DEBUG]')
  })

  it('NO_COLOR env variable disables colors', () => {
    const cmd = `NO_COLOR=1 bun run dist/main.js --debug "echo test" 2>&1`
    let output = ''
    try {
      output = execSync(cmd, { encoding: 'utf-8' })
    } catch (error) {
      // Command may exit with non-zero, but that's ok for this test
    }

    // Should not contain ANSI escape codes
    expect(output).not.toMatch(/\x1b\[/)
  })

  it('truncates log file when it exceeds 10MB', () => {
    // Create a large file
    fs.mkdirSync(tmpDir, { recursive: true })
    const largeContent = 'x'.repeat(11 * 1024 * 1024) // 11MB
    fs.writeFileSync(tmpFile, largeContent)

    const sizeBefore = fs.statSync(tmpFile).size
    expect(sizeBefore).toBeGreaterThan(10 * 1024 * 1024)

    // Run with debug file
    const cmd = `bun run dist/main.js --debug --debug-file ${tmpFile} "echo test" 2>&1`
    try {
      execSync(cmd, { encoding: 'utf-8' })
    } catch (error) {
      // Command may exit with non-zero, but that's ok for this test
    }

    const sizeAfter = fs.statSync(tmpFile).size
    expect(sizeAfter).toBeLessThan(100000) // Much smaller than 10MB
  })
})
