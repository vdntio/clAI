// tests/safety.test.ts
// Tests for the safety module (dangerous command detection)

import { describe, it, expect, beforeEach, afterAll } from 'vitest'
import {
  SafetyError,
  DEFAULT_DANGEROUS_PATTERNS,
  compilePatterns,
  isDangerous,
  loadPatterns,
  shouldPrompt,
  checkSafety,
} from '../src/safety/index.js'
import type { Config } from '../src/config/types.js'

// Helper to create a minimal test config
function createTestConfig(overrides: Partial<Config> = {}): Config {
  return {
    provider: { default: 'openrouter', fallback: [] },
    context: {
      maxFiles: 10,
      maxHistory: 3,
      redactPaths: false,
      redactUsername: false,
    },
    safety: {
      confirmDangerous: true,
      dangerousPatterns: [],
    },
    ui: {
      color: 'auto',
      interactive: false,
      numOptions: 1,
      promptTimeout: 30000,
    },
    providers: {},
    quiet: false,
    verbose: 0,
    force: false,
    dryRun: false,
    offline: false,
    debug: false,
    instruction: 'test',
    ...overrides,
  }
}

describe('SafetyError', () => {
  it('should have exit code 5', () => {
    const error = new SafetyError('Test error')
    expect(error.code).toBe(5)
  })

  it('should have correct name', () => {
    const error = new SafetyError('Test error')
    expect(error.name).toBe('SafetyError')
  })

  it('should store message', () => {
    const error = new SafetyError('User aborted')
    expect(error.message).toBe('User aborted')
  })

  it('should be instanceof Error', () => {
    const error = new SafetyError('Test')
    expect(error instanceof Error).toBe(true)
    expect(error instanceof SafetyError).toBe(true)
  })
})

describe('DEFAULT_DANGEROUS_PATTERNS', () => {
  it('should contain common dangerous patterns', () => {
    expect(DEFAULT_DANGEROUS_PATTERNS.length).toBeGreaterThan(10)
    expect(DEFAULT_DANGEROUS_PATTERNS.some((p) => p.includes('rm'))).toBe(true)
    expect(DEFAULT_DANGEROUS_PATTERNS.some((p) => p.includes('dd'))).toBe(true)
    expect(DEFAULT_DANGEROUS_PATTERNS.some((p) => p.includes('mkfs'))).toBe(true)
  })
})

describe('compilePatterns', () => {
  it('should compile valid patterns', () => {
    const patterns = ['rm\\s+-rf', 'sudo\\s+rm']
    const compiled = compilePatterns(patterns)

    expect(compiled.length).toBe(2)
    expect(compiled[0]?.isValid).toBe(true)
    expect(compiled[0]?.regex).toBeInstanceOf(RegExp)
    expect(compiled[1]?.isValid).toBe(true)
  })

  it('should handle invalid regex patterns', () => {
    const patterns = ['rm -rf', '[invalid('] // Second pattern is invalid regex
    const compiled = compilePatterns(patterns)

    expect(compiled.length).toBe(2)
    expect(compiled[0]?.isValid).toBe(true)
    expect(compiled[1]?.isValid).toBe(false)
    expect(compiled[1]?.regex).toBeNull()
  })

  it('should handle empty array', () => {
    const compiled = compilePatterns([])
    expect(compiled).toEqual([])
  })

  it('should compile with case-insensitive flag', () => {
    const patterns = ['drop\\s+database']
    const compiled = compilePatterns(patterns)

    expect(compiled[0]?.regex?.test('DROP DATABASE')).toBe(true)
    expect(compiled[0]?.regex?.test('drop database')).toBe(true)
  })
})

describe('isDangerous', () => {
  const patterns = compilePatterns(DEFAULT_DANGEROUS_PATTERNS)

  describe('safe commands', () => {
    const safeCommands = [
      'ls -la',
      'cat file.txt',
      'grep "pattern" file.txt',
      'echo "hello world"',
      'pwd',
      'cd /home/user',
      'mkdir -p new_dir',
      'cp file1.txt file2.txt',
      'git status',
      'git commit -m "message"',
      'npm install',
      'node script.js',
    ]

    it.each(safeCommands)('should mark "%s" as safe', (cmd) => {
      expect(isDangerous(cmd, patterns)).toBe(false)
    })
  })

  describe('dangerous commands', () => {
    const dangerousCommands = [
      'rm -rf /',
      'rm -rf /home',
      'rm -f important_file',
      'sudo rm -rf /',
      'dd if=/dev/zero of=/dev/sda',
      'mkfs.ext4 /dev/sda1',
      'DROP DATABASE production',
      'drop table users',
      'git reset --hard HEAD~5',
      'git push --force origin main',
      'chmod 777 /',
      'find . -exec rm -rf {} \\;',
    ]

    it.each(dangerousCommands)('should mark "%s" as dangerous', (cmd) => {
      expect(isDangerous(cmd, patterns)).toBe(true)
    })
  })

  it('should treat empty command as safe', () => {
    expect(isDangerous('', patterns)).toBe(false)
    expect(isDangerous('   ', patterns)).toBe(false)
  })

  it('should fail-safe when pattern is invalid', () => {
    const invalidPatterns = compilePatterns(['[invalid('])
    // With invalid pattern, all commands should be treated as dangerous
    expect(isDangerous('ls', invalidPatterns)).toBe(true)
    expect(isDangerous('safe command', invalidPatterns)).toBe(true)
  })
})

describe('loadPatterns', () => {
  it('should use default patterns when config has empty array', () => {
    const config = createTestConfig()
    const patterns = loadPatterns(config)

    expect(patterns.length).toBe(DEFAULT_DANGEROUS_PATTERNS.length)
  })

  it('should use custom patterns from config', () => {
    const config = createTestConfig({
      safety: {
        confirmDangerous: true,
        dangerousPatterns: ['custom-pattern', 'another-pattern'],
      },
    })
    const patterns = loadPatterns(config)

    expect(patterns.length).toBe(2)
    expect(patterns[0]?.pattern).toBe('custom-pattern')
  })
})

describe('shouldPrompt', () => {
  // Save original TTY values
  const originalStdinTTY = process.stdin.isTTY
  const originalStdoutTTY = process.stdout.isTTY

  beforeEach(() => {
    // Reset to TTY mode for tests
    Object.defineProperty(process.stdin, 'isTTY', { value: true, writable: true })
    Object.defineProperty(process.stdout, 'isTTY', { value: true, writable: true })
  })

  // Restore after all tests
  afterAll(() => {
    Object.defineProperty(process.stdin, 'isTTY', { value: originalStdinTTY })
    Object.defineProperty(process.stdout, 'isTTY', { value: originalStdoutTTY })
  })

  it('should return true when all conditions are met', () => {
    const config = createTestConfig()
    expect(shouldPrompt(config)).toBe(true)
  })

  it('should return false when confirmDangerous is disabled', () => {
    const config = createTestConfig({
      safety: { confirmDangerous: false, dangerousPatterns: [] },
    })
    expect(shouldPrompt(config)).toBe(false)
  })

  it('should return false when force flag is set', () => {
    const config = createTestConfig({ force: true })
    expect(shouldPrompt(config)).toBe(false)
  })

  it('should return false in non-TTY mode (piped)', () => {
    Object.defineProperty(process.stdin, 'isTTY', { value: false })
    const config = createTestConfig()
    expect(shouldPrompt(config)).toBe(false)
  })
})

describe('checkSafety', () => {
  // Save original TTY values
  const originalStdinTTY = process.stdin.isTTY
  const originalStdoutTTY = process.stdout.isTTY

  beforeEach(() => {
    // Reset to TTY mode for tests
    Object.defineProperty(process.stdin, 'isTTY', { value: true, writable: true })
    Object.defineProperty(process.stdout, 'isTTY', { value: true, writable: true })
  })

  afterAll(() => {
    Object.defineProperty(process.stdin, 'isTTY', { value: originalStdinTTY })
    Object.defineProperty(process.stdout, 'isTTY', { value: originalStdoutTTY })
  })

  it('should detect dangerous command and require prompt', () => {
    const config = createTestConfig()
    const result = checkSafety(['rm -rf /'], config)

    expect(result.isDangerous).toBe(true)
    expect(result.shouldPrompt).toBe(true)
  })

  it('should not prompt for safe commands', () => {
    const config = createTestConfig()
    const result = checkSafety(['ls -la'], config)

    expect(result.isDangerous).toBe(false)
    expect(result.shouldPrompt).toBe(false)
  })

  it('should detect dangerous in array of mixed commands', () => {
    const config = createTestConfig()
    const result = checkSafety(['ls -la', 'rm -rf /', 'cat file'], config)

    expect(result.isDangerous).toBe(true)
    expect(result.shouldPrompt).toBe(true)
  })

  it('should not prompt when force is set', () => {
    const config = createTestConfig({ force: true })
    const result = checkSafety(['rm -rf /'], config)

    expect(result.isDangerous).toBe(true)
    expect(result.shouldPrompt).toBe(false)
  })
})
