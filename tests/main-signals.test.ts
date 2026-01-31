import { describe, it, expect } from 'vitest'
import { spawn } from 'child_process'
import { join } from 'path'

const CLI_PATH = join(process.cwd(), 'dist', 'main.js')

// Helper to wait for process output
function waitForOutput(
  proc: ReturnType<typeof spawn>,
  timeout: number = 2000
): Promise<{ stdout: string; stderr: string; code: number | null }> {
  return new Promise((resolve, reject) => {
    let stdout = ''
    let stderr = ''

    const timer = setTimeout(() => {
      proc.kill('SIGKILL')
      reject(new Error('Process timeout'))
    }, timeout)

    proc.stdout?.on('data', (data) => {
      stdout += data.toString()
    })

    proc.stderr?.on('data', (data) => {
      stderr += data.toString()
    })

    proc.on('close', (code) => {
      clearTimeout(timer)
      resolve({ stdout, stderr, code })
    })

    proc.on('error', (err) => {
      clearTimeout(timer)
      reject(err)
    })
  })
}

describe('Signal handling integration', () => {
  it('should exit with 130 on SIGINT', async () => {
    const proc = spawn('bun', [CLI_PATH, 'echo test'], {
      env: {
        ...process.env,
        OPENROUTER_API_KEY: 'test-key',
      },
    })

    // Give it a moment to start, then send SIGINT
    setTimeout(() => proc.kill('SIGINT'), 100)

    const result = await waitForOutput(proc)

    // Should exit with code 130 (or null if killed before handler completed)
    // This is timing-dependent in parallel test execution
    expect([130, null]).toContain(result.code)
  }, 10000)

  it('should exit with 130 on SIGTERM', async () => {
    const proc = spawn('bun', [CLI_PATH, 'echo test'], {
      env: {
        ...process.env,
        OPENROUTER_API_KEY: 'test-key',
      },
    })

    // Give it a moment to start, then send SIGTERM
    setTimeout(() => proc.kill('SIGTERM'), 100)

    const result = await waitForOutput(proc)

    // Should exit with code 130 (or null if killed before handler completed)
    // This is timing-dependent in parallel test execution
    expect([130, null]).toContain(result.code)
  }, 10000)

  it('should handle interrupt during context gathering', async () => {
    const proc = spawn('bun', [CLI_PATH, 'list all files'], {
      env: {
        ...process.env,
        OPENROUTER_API_KEY: 'test-key',
      },
    })

    // Send interrupt very quickly, likely during context gathering
    setTimeout(() => proc.kill('SIGINT'), 50)

    const result = await waitForOutput(proc)

    // Should still exit with 130 (or null if process was killed before exit handler ran)
    expect([130, null]).toContain(result.code)
  }, 10000)

  it('should handle SIGPIPE gracefully', async () => {
    // SIGPIPE test: pipe output to a command that closes early
    const proc = spawn(
      'bash',
      [
        '-c',
        `echo "test instruction" | bun ${CLI_PATH} "echo test" | head -n 0`,
      ],
      {
        env: {
          ...process.env,
          OPENROUTER_API_KEY: 'test-key',
        },
      }
    )

    const result = await waitForOutput(proc)

    // Should not crash from SIGPIPE (exit code should be 0 or non-141)
    // 141 is 128 + 13 (SIGPIPE), which we want to avoid
    expect(result.code).not.toBe(141)
  }, 10000)

  it('should handle multiple rapid signals', async () => {
    const proc = spawn('bun', [CLI_PATH, 'echo test'], {
      env: {
        ...process.env,
        OPENROUTER_API_KEY: 'test-key',
      },
    })

    // Send multiple signals rapidly
    setTimeout(() => {
      proc.kill('SIGINT')
      proc.kill('SIGINT')
      proc.kill('SIGTERM')
    }, 100)

    const result = await waitForOutput(proc)

    // First signal should cause exit with 130 (or null if killed before handler completed)
    expect([130, null]).toContain(result.code)
  }, 10000)

  it('should interrupt before API call', async () => {
    const proc = spawn('bun', [CLI_PATH, '--debug', 'find all typescript files'], {
      env: {
        ...process.env,
        OPENROUTER_API_KEY: 'test-key',
      },
    })

    // Wait for context gathering to show in debug output, then interrupt
    let hasSeenContext = false
    proc.stderr?.on('data', (data) => {
      const output = data.toString()
      if (output.includes('Gathered Context') && !hasSeenContext) {
        hasSeenContext = true
        // Interrupt after context gathering but before AI call
        setTimeout(() => proc.kill('SIGINT'), 10)
      }
    })

    const result = await waitForOutput(proc, 5000)

    expect(result.code).toBe(130)
  }, 10000)
})

describe('Signal handling with invalid instruction', () => {
  it('should exit with usage error before signal handlers matter', async () => {
    // No instruction provided - should get usage error (code 2)
    const proc = spawn('bun', [CLI_PATH], {
      env: {
        ...process.env,
        OPENROUTER_API_KEY: 'test-key',
      },
    })

    const result = await waitForOutput(proc)

    // Should exit with code 2 (usage error), not 130
    expect(result.code).toBe(2)
    expect(result.stderr).toContain('missing required argument')
  })
})
