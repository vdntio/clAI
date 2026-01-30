// Integration tests for main.ts error handling
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { spawn } from 'child_process'
import { resolve } from 'path'

const CLI_PATH = resolve(__dirname, '../dist/main.js')

describe('main.ts error handling', () => {
  it('should exit with code 2 for UsageError (missing instruction)', (done) => {
    const proc = spawn('bun', [CLI_PATH], {
      stdio: ['pipe', 'pipe', 'pipe'],
    })

    let stderr = ''
    proc.stderr.on('data', (chunk) => {
      stderr += chunk.toString()
    })

    proc.on('close', (code) => {
      expect(code).toBe(2)
      expect(stderr).toContain('missing required argument')
      expect(stderr).toContain("Try 'clai --help'")
      done()
    })
  })

  it('should exit with code 4 for AIError (missing API key)', (done) => {
    const proc = spawn('bun', [CLI_PATH, 'test command'], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {
        ...process.env,
        OPENROUTER_API_KEY: '', // Clear API key
        MOCK_AI: '0', // Disable mock mode
      },
    })

    let stderr = ''
    proc.stderr.on('data', (chunk) => {
      stderr += chunk.toString()
    })

    proc.on('close', (code) => {
      expect(code).toBe(4)
      expect(stderr).toContain('OpenRouter API key not configured')
      done()
    })
  })

  it('should exit with code 0 for --help', (done) => {
    const proc = spawn('bun', [CLI_PATH, '--help'], {
      stdio: ['pipe', 'pipe', 'pipe'],
    })

    proc.on('close', (code) => {
      expect(code).toBe(0)
      done()
    })
  })

  it('should exit with code 0 for --version', (done) => {
    const proc = spawn('bun', [CLI_PATH, '--version'], {
      stdio: ['pipe', 'pipe', 'pipe'],
    })

    proc.on('close', (code) => {
      expect(code).toBe(0)
      done()
    })
  })
})
