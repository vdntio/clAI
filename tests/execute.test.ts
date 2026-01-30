// tests/execute.test.ts
// Tests for the command execution module

import { describe, it, expect, afterEach } from 'vitest'
import {
  ExecutionError,
  Errors,
  isRecursiveCall,
  validateCommand,
  getShell,
  executeCommand,
} from '../src/output/index.js'

describe('ExecutionError', () => {
  it('should have the correct name', () => {
    const error = new ExecutionError('test', 1)
    expect(error.name).toBe('ExecutionError')
  })

  it('should store the exit code', () => {
    const error = new ExecutionError('test', 42)
    expect(error.code).toBe(42)
  })

  it('should store the message', () => {
    const error = new ExecutionError('Something went wrong', 1)
    expect(error.message).toBe('Something went wrong')
  })

  it('should store the cause if provided', () => {
    const cause = new Error('Original error')
    const error = new ExecutionError('Wrapped', 1, cause)
    expect(error.cause).toBe(cause)
  })

  it('should be instanceof Error', () => {
    const error = new ExecutionError('test', 1)
    expect(error instanceof Error).toBe(true)
    expect(error instanceof ExecutionError).toBe(true)
  })
})

describe('Errors factory functions', () => {
  it('emptyCommand returns error with code 1', () => {
    const error = Errors.emptyCommand()
    expect(error.code).toBe(1)
    expect(error.message).toBe('Empty command')
  })

  it('spawnFailed returns error with code 1 and cause', () => {
    const cause = new Error('ENOENT')
    const error = Errors.spawnFailed('ls', cause)
    expect(error.code).toBe(1)
    expect(error.message).toBe('Failed to spawn: ls')
    expect(error.cause).toBe(cause)
  })

  it('shellNotFound returns error with code 127', () => {
    const error = Errors.shellNotFound('/bin/nonexistent')
    expect(error.code).toBe(127)
    expect(error.message).toBe('Shell not found: /bin/nonexistent')
  })

  it('commandNotFound returns error with code 127', () => {
    const error = Errors.commandNotFound('nonexistentcmd')
    expect(error.code).toBe(127)
    expect(error.message).toBe('Command not found: nonexistentcmd')
  })

  it('permissionDenied returns error with code 126', () => {
    const error = Errors.permissionDenied('restricted-cmd')
    expect(error.code).toBe(126)
    expect(error.message).toBe('Permission denied: restricted-cmd')
  })

  it('signalKilled returns error with code 128', () => {
    const error = Errors.signalKilled('SIGTERM')
    expect(error.code).toBe(128)
    expect(error.message).toBe('Killed by signal: SIGTERM')
  })

  it('timeout returns error with code 124', () => {
    const error = Errors.timeout(5000)
    expect(error.code).toBe(124)
    expect(error.message).toBe('Command timed out after 5000ms')
  })

  it('recursiveCall returns error with code 5', () => {
    const error = Errors.recursiveCall()
    expect(error.code).toBe(5)
    expect(error.message).toContain('recursive')
    expect(error.message).toContain('infinite loop')
  })
})

describe('getShell', () => {
  const originalShell = process.env.SHELL

  afterEach(() => {
    if (originalShell === undefined) {
      delete process.env.SHELL
    } else {
      process.env.SHELL = originalShell
    }
  })

  it('returns SHELL env var when set', () => {
    process.env.SHELL = '/bin/zsh'
    expect(getShell()).toBe('/bin/zsh')
  })

  it('falls back to /bin/sh when SHELL not set', () => {
    delete process.env.SHELL
    expect(getShell()).toBe('/bin/sh')
  })
})

describe('isRecursiveCall', () => {
  describe('should detect recursive calls', () => {
    it('detects "clai" at start', () => {
      expect(isRecursiveCall('clai "find files"')).toBe(true)
      expect(isRecursiveCall('clai')).toBe(true)
      expect(isRecursiveCall('clai --help')).toBe(true)
    })

    it('detects "./clai" at start', () => {
      expect(isRecursiveCall('./clai "list files"')).toBe(true)
      expect(isRecursiveCall('./clai')).toBe(true)
    })

    it('detects "/path/to/clai" at start', () => {
      expect(isRecursiveCall('/usr/local/bin/clai "help"')).toBe(true)
      expect(isRecursiveCall('/home/user/bin/clai foo')).toBe(true)
    })

    it('detects "clai" after pipe', () => {
      expect(isRecursiveCall('echo "test" | clai')).toBe(true)
      expect(isRecursiveCall('cat file.txt | clai "analyze"')).toBe(true)
      expect(isRecursiveCall('ls |clai')).toBe(true)
    })

    it('detects "clai" after &&', () => {
      expect(isRecursiveCall('cd foo && clai "list"')).toBe(true)
      expect(isRecursiveCall('mkdir test && clai run')).toBe(true)
    })

    it('detects "clai" after semicolon', () => {
      expect(isRecursiveCall('echo hi; clai "help"')).toBe(true)
      expect(isRecursiveCall('pwd;clai')).toBe(true)
    })
  })

  describe('should NOT flag false positives', () => {
    it('allows "claimant" (clai is substring)', () => {
      expect(isRecursiveCall('claimant')).toBe(false)
      expect(isRecursiveCall('echo claimant')).toBe(false)
    })

    it('allows "claiming" (clai is prefix of longer word)', () => {
      expect(isRecursiveCall('claiming')).toBe(false)
    })

    it('allows "echo clai" (clai is argument, not command)', () => {
      expect(isRecursiveCall('echo clai')).toBe(false)
      expect(isRecursiveCall('echo "clai is cool"')).toBe(false)
    })

    it('allows "/usr/bin/claim" (different command)', () => {
      expect(isRecursiveCall('/usr/bin/claim')).toBe(false)
      expect(isRecursiveCall('claim --version')).toBe(false)
    })

    it('allows "git clone" and other commands', () => {
      expect(isRecursiveCall('git clone https://example.com')).toBe(false)
      expect(isRecursiveCall('ls -la')).toBe(false)
      expect(isRecursiveCall('find . -name "*.ts"')).toBe(false)
    })

    it('allows "xclaim" (different command)', () => {
      expect(isRecursiveCall('xclaim foo')).toBe(false)
    })
  })
})

describe('validateCommand', () => {
  it('returns valid for normal commands', () => {
    expect(validateCommand('ls -la')).toEqual({ valid: true })
    expect(validateCommand('echo "hello"')).toEqual({ valid: true })
    expect(validateCommand('git status')).toEqual({ valid: true })
  })

  it('returns error for empty command', () => {
    const result = validateCommand('')
    expect(result.valid).toBe(false)
    if (!result.valid) {
      expect(result.error.code).toBe(1)
      expect(result.error.message).toBe('Empty command')
    }
  })

  it('returns error for whitespace-only command', () => {
    const result = validateCommand('   ')
    expect(result.valid).toBe(false)
    if (!result.valid) {
      expect(result.error.code).toBe(1)
    }
  })

  it('returns error for recursive clai call', () => {
    const result = validateCommand('clai "help me"')
    expect(result.valid).toBe(false)
    if (!result.valid) {
      expect(result.error.code).toBe(5)
      expect(result.error.message).toContain('recursive')
    }
  })

  it('returns error for piped recursive call', () => {
    const result = validateCommand('echo test | clai')
    expect(result.valid).toBe(false)
    if (!result.valid) {
      expect(result.error.code).toBe(5)
    }
  })
})

describe('executeCommand', () => {
  describe('success cases', () => {
    it('executes simple command and returns exit 0', async () => {
      const result = await executeCommand('true', { inheritStdio: false })
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.exitCode).toBe(0)
      }
    })

    it('propagates non-zero exit codes', async () => {
      const result = await executeCommand('false', { inheritStdio: false })
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.exitCode).toBe(1)
      }
    })

    it('propagates exit code 42', async () => {
      const result = await executeCommand('exit 42', { inheritStdio: false })
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.exitCode).toBe(42)
      }
    })

    it('handles commands with special characters', async () => {
      const result = await executeCommand('echo "hello world"', { inheritStdio: false })
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.exitCode).toBe(0)
      }
    })

    it('handles commands with quotes', async () => {
      const result = await executeCommand("echo 'single' \"double\"", { inheritStdio: false })
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.exitCode).toBe(0)
      }
    })

    it('handles commands with shell variables', async () => {
      const result = await executeCommand('echo $HOME', { inheritStdio: false })
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.exitCode).toBe(0)
      }
    })
  })

  describe('error cases', () => {
    it('returns error for empty command', async () => {
      const result = await executeCommand('')
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.code).toBe(1)
        expect(result.error.message).toBe('Empty command')
      }
    })

    it('returns error code 127 for command not found', async () => {
      const result = await executeCommand('nonexistentcmd123xyz', { inheritStdio: false })
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.code).toBe(127)
      }
    })

    it('returns error code 5 for recursive "clai" command', async () => {
      const result = await executeCommand('clai "find files"')
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.code).toBe(5)
        expect(result.error.message).toContain('recursive')
      }
    })

    it('returns error code 5 for "./clai foo"', async () => {
      const result = await executeCommand('./clai foo')
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.code).toBe(5)
      }
    })

    it('returns error code 5 for "ls | clai bar"', async () => {
      const result = await executeCommand('ls | clai bar')
      expect(result.success).toBe(false)
      if (!result.success) {
        expect(result.error.code).toBe(5)
      }
    })
  })

  describe('edge cases', () => {
    it('handles very long commands', async () => {
      const longArg = 'x'.repeat(1000)
      const result = await executeCommand(`echo "${longArg}"`, { inheritStdio: false })
      expect(result.success).toBe(true)
      if (result.success) {
        expect(result.exitCode).toBe(0)
      }
    })

    it('uses specified shell', async () => {
      const result = await executeCommand('echo test', {
        shell: '/bin/sh',
        inheritStdio: false,
      })
      expect(result.success).toBe(true)
    })
  })
})

describe('executeCommand integration', () => {
  it('runs echo and captures exit code', async () => {
    const result = await executeCommand('echo "hello world"', { inheritStdio: false })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.exitCode).toBe(0)
    }
  })

  it('runs false and gets exit code 1', async () => {
    const result = await executeCommand('false', { inheritStdio: false })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.exitCode).toBe(1)
    }
  })

  it('runs exit 42 and gets exit code 42', async () => {
    const result = await executeCommand('exit 42', { inheritStdio: false })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.exitCode).toBe(42)
    }
  })

  it('runs nonexistent command and gets error 127', async () => {
    const result = await executeCommand('__definitely_not_a_real_command_xyz__', {
      inheritStdio: false,
    })
    expect(result.success).toBe(false)
    if (!result.success) {
      expect(result.error.code).toBe(127)
    }
  })

  it('handles multiline commands', async () => {
    const result = await executeCommand('echo line1\necho line2', { inheritStdio: false })
    expect(result.success).toBe(true)
  })

  it('handles piped commands', async () => {
    const result = await executeCommand('echo "hello" | cat', { inheritStdio: false })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.exitCode).toBe(0)
    }
  })

  it('handles command with environment variable', async () => {
    process.env.TEST_EXECUTE_VAR = 'test_value'
    const result = await executeCommand('echo $TEST_EXECUTE_VAR', { inheritStdio: false })
    expect(result.success).toBe(true)
    if (result.success) {
      expect(result.exitCode).toBe(0)
    }
    delete process.env.TEST_EXECUTE_VAR
  })
})
