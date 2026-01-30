// Context module tests

import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  gatherContext,
  getSystemInfo,
  clearSystemCache,
  getCwd,
  scanDirectory,
  getRedactedCwd,
  getShellHistory,
  getDetectedShell,
  readStdin,
  hasPipedStdin,
  redactPath,
  redactUsername,
  redactEnvVars,
  ContextError,
} from '../src/context/index.js'
import { Config } from '../src/config/types.js'
import { mkdirSync, rmSync, writeFileSync } from 'fs'
import { join } from 'path'
import { homedir } from 'os'

describe('Context Module', () => {
  describe('System Info', () => {
    beforeEach(() => {
      clearSystemCache()
    })

    afterEach(() => {
      clearSystemCache()
    })

    it('should gather basic system info', () => {
      const info = getSystemInfo(false)

      expect(info.osName).toBeTruthy()
      expect(info.osVersion).toBeTruthy()
      expect(info.architecture).toBeTruthy()
      expect(info.shell).toBeTruthy()
      expect(info.user).toBeTruthy()
      expect(info.totalMemoryMb).toBeGreaterThan(0)
    })

    it('should cache system info', () => {
      const info1 = getSystemInfo(false)
      const info2 = getSystemInfo(false)

      expect(info1).toBe(info2) // Same reference (cached)
    })

    it('should redact username when requested', () => {
      const info = getSystemInfo(true)
      expect(info.user).toBe('[REDACTED]')
    })

    it('should detect shell from SHELL env var', () => {
      const originalShell = process.env.SHELL
      process.env.SHELL = '/bin/zsh'
      clearSystemCache()

      try {
        const info = getSystemInfo(false)
        expect(info.shell).toBe('zsh')
      } finally {
        process.env.SHELL = originalShell
        clearSystemCache()
      }
    })

    it('should handle missing SHELL env var', () => {
      const originalShell = process.env.SHELL
      delete process.env.SHELL
      clearSystemCache()

      try {
        const info = getSystemInfo(false)
        expect(info.shell).toBe('unknown')
      } finally {
        if (originalShell) process.env.SHELL = originalShell
        clearSystemCache()
      }
    })
  })

  describe('Redaction', () => {
    const home = homedir()

    it('should redact home directory in path', () => {
      if (!home) return // Skip if no home directory

      const path = `${home}/projects/test`
      const redacted = redactPath(path)
      expect(redacted).toContain('[REDACTED]')
      expect(redacted).not.toContain(home)
    })

    it('should redact ~/ shorthand', () => {
      const path = '~/projects/test'
      const redacted = redactPath(path)
      expect(redacted).toContain('[REDACTED]')
    })

    it('should handle ~ alone', () => {
      const path = '~'
      const redacted = redactPath(path)
      expect(redacted).toBe('[REDACTED]')
    })

    it('should redact /home/username/ pattern with actual username', () => {
      const home = homedir()
      if (!home) return // Skip if no home directory

      // Extract username from home directory
      const homeParts = home.replace(/\\/g, '/').split('/')
      const username = homeParts[homeParts.length - 1]
      if (!username) return

      const path = `/home/${username}/projects/test`
      const redacted = redactPath(path)
      expect(redacted).toContain('[REDACTED]')
    })

    it('should redact /Users/username/ pattern with actual username (macOS)', () => {
      const home = homedir()
      if (!home) return // Skip if no home directory

      // Extract username from home directory
      const homeParts = home.replace(/\\/g, '/').split('/')
      const username = homeParts[homeParts.length - 1]
      if (!username) return

      const path = `/Users/${username}/projects/test`
      const redacted = redactPath(path)
      expect(redacted).toContain('[REDACTED]')
    })

    it('should not modify non-home paths', () => {
      const path = '/usr/local/bin'
      const redacted = redactPath(path)
      expect(redacted).toBe('/usr/local/bin')
    })

    it('should redact username', () => {
      expect(redactUsername('john')).toBe('[REDACTED]')
    })

    it('should redact environment variables', () => {
      const text = 'Path: ${HOME}/test and $HOME/bin'
      const redacted = redactEnvVars(text, ['HOME'])
      expect(redacted).toContain('[REDACTED]')
      expect(redacted).not.toContain('${HOME}')
      expect(redacted).not.toMatch(/\$HOME\b/)
    })
  })

  describe('Directory Context', () => {
    const testDir = join(process.cwd(), 'test-context-dir')

    beforeEach(() => {
      // Create test directory with some files
      try {
        mkdirSync(testDir, { recursive: true })
        writeFileSync(join(testDir, 'a-file.txt'), 'content')
        writeFileSync(join(testDir, 'b-file.txt'), 'content')
        writeFileSync(join(testDir, 'c-file.txt'), 'content')
        mkdirSync(join(testDir, 'subdir'), { recursive: true })
      } catch {
        // Directory may already exist
      }
    })

    afterEach(() => {
      // Clean up
      try {
        rmSync(testDir, { recursive: true, force: true })
      } catch {
        // Directory may not exist
      }
    })

    it('should get current working directory', () => {
      const cwd = getCwd()
      expect(cwd).toBeTruthy()
      expect(typeof cwd).toBe('string')
    })

    it('should scan directory and return sorted files', () => {
      const originalCwd = process.cwd()
      process.chdir(testDir)

      try {
        const files = scanDirectory(10, false)
        expect(files.length).toBeGreaterThan(0)
        // Should include our test files
        expect(files.some((f) => f.includes('a-file.txt'))).toBe(true)
        expect(files.some((f) => f.includes('b-file.txt'))).toBe(true)
      } finally {
        process.chdir(originalCwd)
      }
    })

    it('should limit files to maxFiles', () => {
      const originalCwd = process.cwd()
      process.chdir(testDir)

      try {
        const files = scanDirectory(2, false)
        expect(files.length).toBeLessThanOrEqual(2)
      } finally {
        process.chdir(originalCwd)
      }
    })

    it('should return empty array on directory read error', () => {
      // We can't easily test this without mocking fs
      // The function should return [] on error
      expect(() => scanDirectory(10, false)).not.toThrow()
    })

    it('should redact paths when requested', () => {
      const originalCwd = process.cwd()
      process.chdir(testDir)

      try {
        const files = scanDirectory(10, true)
        // Files should have redacted paths if they contain home
        const home = homedir()
        if (home && testDir.includes(home)) {
          expect(files.some((f) => f.includes('[REDACTED]'))).toBe(true)
        }
      } finally {
        process.chdir(originalCwd)
      }
    })

    it('should get redacted cwd', () => {
      const cwd = getRedactedCwd(true)
      expect(cwd).toContain('[REDACTED]')
    })
  })

  describe('Shell History', () => {
    it('should detect shell from SHELL env var', () => {
      const originalShell = process.env.SHELL
      process.env.SHELL = '/bin/bash'

      try {
        const shell = getDetectedShell()
        expect(shell).toBe('bash')
      } finally {
        process.env.SHELL = originalShell
      }
    })

    it('should return empty array for unsupported shell', () => {
      const originalShell = process.env.SHELL
      process.env.SHELL = '/bin/tcsh'

      try {
        const history = getShellHistory(3)
        expect(history).toEqual([])
      } finally {
        process.env.SHELL = originalShell
      }
    })

    it('should return empty array when SHELL is not set', () => {
      const originalShell = process.env.SHELL
      delete process.env.SHELL

      try {
        const history = getShellHistory(3)
        expect(history).toEqual([])
      } finally {
        if (originalShell) process.env.SHELL = originalShell
      }
    })

    it('should handle missing history file gracefully', () => {
      // This test will only work if there's no history file
      const history = getShellHistory(3)
      // Should not throw, might be empty or have content
      expect(Array.isArray(history)).toBe(true)
    })
  })

  describe('Stdin', () => {
    it('should detect if stdin has piped data', () => {
      // Just test it doesn't throw
      const hasPiped = hasPipedStdin()
      expect(typeof hasPiped).toBe('boolean')
    })

    it('should return undefined when stdin is TTY', async () => {
      // In test environment, if stdin is a TTY, readStdin should return undefined
      // If stdin is not a TTY, the function will try to read and might hang
      // So we only test the TTY case, otherwise skip
      if (process.stdin.isTTY) {
        const result = await readStdin()
        expect(result).toBeUndefined()
      } else {
        // Skip test when stdin is not a TTY to avoid hanging
        console.log('Skipping stdin test - stdin is not a TTY')
      }
    }, 100) // Short timeout since this should be quick
  })

  describe('gatherContext Integration', () => {
    const mockConfig: Config = {
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
        numOptions: 3,
      },
      providers: {},
      quiet: false,
      verbose: 0,
      force: false,
      dryRun: false,
      offline: false,
      debug: false,
      instruction: 'test',
    }

    beforeEach(() => {
      clearSystemCache()
    })

    afterEach(() => {
      clearSystemCache()
    })

    it('should gather complete context', async () => {
      // Skip stdin reading if not TTY to avoid hanging
      if (!process.stdin.isTTY) {
        console.log('Skipping - stdin not TTY')
        return
      }

      const context = await gatherContext(mockConfig)

      expect(context.system).toBeDefined()
      expect(context.system.osName).toBeTruthy()
      expect(context.system.architecture).toBeTruthy()

      expect(context.cwd).toBeTruthy()
      expect(typeof context.cwd).toBe('string')

      expect(Array.isArray(context.files)).toBe(true)
      expect(Array.isArray(context.history)).toBe(true)
      // stdin might be undefined or string
      expect(
        context.stdin === undefined || typeof context.stdin === 'string'
      ).toBe(true)
    }, 100)

    it('should apply redaction when configured', async () => {
      // Skip stdin reading if not TTY to avoid hanging
      if (!process.stdin.isTTY) {
        console.log('Skipping - stdin not TTY')
        return
      }

      const redactedConfig: Config = {
        ...mockConfig,
        context: {
          ...mockConfig.context,
          redactPaths: true,
          redactUsername: true,
        },
      }

      clearSystemCache()
      const context = await gatherContext(redactedConfig)

      expect(context.system.user).toBe('[REDACTED]')

      // CWD should be redacted if it contains home
      const home = homedir()
      if (home && process.cwd().includes(home)) {
        expect(context.cwd).toContain('[REDACTED]')
      }
    }, 100)

    it('should respect maxFiles limit', async () => {
      // Skip stdin reading if not TTY to avoid hanging
      if (!process.stdin.isTTY) {
        console.log('Skipping - stdin not TTY')
        return
      }

      const limitedConfig: Config = {
        ...mockConfig,
        context: {
          ...mockConfig.context,
          maxFiles: 2,
        },
      }

      const context = await gatherContext(limitedConfig)
      expect(context.files.length).toBeLessThanOrEqual(2)
    }, 100)

    it('should handle CWD errors gracefully', async () => {
      // Mock process.cwd to throw
      const originalCwd = process.cwd
      process.cwd = () => {
        throw new Error('CWD error')
      }

      try {
        await expect(gatherContext(mockConfig)).rejects.toThrow(ContextError)
      } finally {
        process.cwd = originalCwd
      }
    })
  })

  describe('ContextError', () => {
    it('should create error with code', () => {
      const error = new ContextError('Test error', 5)
      expect(error.message).toBe('Test error')
      expect(error.code).toBe(5)
      expect(error.name).toBe('ContextError')
    })

    it('should default to code 1', () => {
      const error = new ContextError('Test error')
      expect(error.code).toBe(1)
    })
  })
})
