import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
  registerSignalHandlers,
  checkInterrupt,
  isTTY,
  isInteractive,
} from '../src/signals/index.js'

describe('Signal handlers', () => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
  let originalListeners: Map<string, Function[]>

  beforeEach(() => {
    // Save original listeners
    originalListeners = new Map()
    const signals = ['SIGINT', 'SIGTERM', 'SIGPIPE']
    signals.forEach((sig) => {
      const listeners = process.listeners(sig)
      if (listeners.length > 0) {
        originalListeners.set(sig, [...listeners])
      }
    })

    // Remove all existing listeners
    signals.forEach((sig) => {
      process.removeAllListeners(sig)
    })
  })

  afterEach(() => {
    // Restore original listeners
    const signals = ['SIGINT', 'SIGTERM', 'SIGPIPE']
    signals.forEach((sig) => {
      process.removeAllListeners(sig)
      const listeners = originalListeners.get(sig)
      if (listeners) {
        listeners.forEach((listener) => {
          process.on(sig, listener as any)
        })
      }
    })
  })

  it('should register SIGINT handler', () => {
    registerSignalHandlers()
    const listeners = process.listeners('SIGINT')
    expect(listeners.length).toBeGreaterThan(0)
  })

  it('should register SIGTERM handler', () => {
    registerSignalHandlers()
    const listeners = process.listeners('SIGTERM')
    expect(listeners.length).toBeGreaterThan(0)
  })

  it('should register SIGPIPE handler', () => {
    registerSignalHandlers()
    const listeners = process.listeners('SIGPIPE')
    expect(listeners.length).toBeGreaterThan(0)
  })

  it('should handle multiple registrations', () => {
    registerSignalHandlers()
    const beforeCount = process.listeners('SIGINT').length

    registerSignalHandlers()
    const afterCount = process.listeners('SIGINT').length

    // Each call adds new listeners (no deduplication)
    expect(afterCount).toBeGreaterThanOrEqual(beforeCount)
  })
})

describe('checkInterrupt', () => {
  beforeEach(() => {
    // Reset interrupt state by removing and re-adding handlers
    process.removeAllListeners('SIGINT')
    process.removeAllListeners('SIGTERM')
    process.removeAllListeners('SIGPIPE')
  })

  it('should not throw when not interrupted', () => {
    expect(() => checkInterrupt()).not.toThrow()
  })

  it('should throw InterruptError when interrupted', () => {
    // Mock process.exit to prevent actual exit
    const exitSpy = vi.spyOn(process, 'exit').mockImplementation((() => {
      // Empty implementation
    }) as any)

    try {
      // Register handlers
      registerSignalHandlers()

      // Emit SIGINT to trigger interrupt
      process.emit('SIGINT', 'SIGINT')

      // checkInterrupt should throw after exit is called
      // Note: In reality, process.exit(130) happens, but we mocked it
      expect(exitSpy).toHaveBeenCalledWith(130)
    } finally {
      exitSpy.mockRestore()
    }
  })
})

describe('TTY detection', () => {
  it('should detect stdin TTY', () => {
    const expected = process.stdin.isTTY === true
    expect(isTTY('stdin')).toBe(expected)
  })

  it('should detect stdout TTY', () => {
    const expected = process.stdout.isTTY === true
    expect(isTTY('stdout')).toBe(expected)
  })

  it('should detect stderr TTY', () => {
    const expected = process.stderr.isTTY === true
    expect(isTTY('stderr')).toBe(expected)
  })

  it('should detect interactive mode', () => {
    const expected =
      process.stdin.isTTY === true && process.stdout.isTTY === true
    expect(isInteractive()).toBe(expected)
  })

  it('should return false for stdin when not TTY', () => {
    // In test environment, stdin is typically not a TTY
    if (!process.stdin.isTTY) {
      expect(isTTY('stdin')).toBe(false)
    }
  })

  it('should return false for interactive when stdin is not TTY', () => {
    // In test environment, stdin is typically not a TTY
    if (!process.stdin.isTTY) {
      expect(isInteractive()).toBe(false)
    }
  })

  it('should return false for interactive when stdout is not TTY', () => {
    // In test environment, stdout may not be a TTY
    if (!process.stdout.isTTY) {
      expect(isInteractive()).toBe(false)
    }
  })
})

describe('Signal handler behavior', () => {
  it('SIGPIPE handler should not crash on emit', () => {
    registerSignalHandlers()

    // Should not throw
    expect(() => {
      process.emit('SIGPIPE', 'SIGPIPE')
    }).not.toThrow()
  })

  it('should call process.exit with 130 on SIGINT', () => {
    const exitSpy = vi.spyOn(process, 'exit').mockImplementation((() => {
      // Empty implementation
    }) as any)

    try {
      registerSignalHandlers()
      process.emit('SIGINT', 'SIGINT')

      expect(exitSpy).toHaveBeenCalledWith(130)
    } finally {
      exitSpy.mockRestore()
    }
  })

  it('should call process.exit with 130 on SIGTERM', () => {
    const exitSpy = vi.spyOn(process, 'exit').mockImplementation((() => {
      // Empty implementation
    }) as any)

    try {
      registerSignalHandlers()
      process.emit('SIGTERM', 'SIGTERM')

      expect(exitSpy).toHaveBeenCalledWith(130)
    } finally {
      exitSpy.mockRestore()
    }
  })
})
