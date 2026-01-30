import { describe, it, expect } from 'vitest'
import { ClaiError, UsageError, InterruptError } from '../src/error/index.js'
import { ConfigError } from '../src/config/index.js'
import { ContextError } from '../src/context/types.js'
import { AIError } from '../src/ai/types.js'
import { SafetyError } from '../src/safety/types.js'
import { ExecutionError } from '../src/output/types.js'

describe('ClaiError', () => {
  it('should have exit code and message', () => {
    const error = new ClaiError('test', 42)
    expect(error.message).toBe('test')
    expect(error.code).toBe(42)
    expect(error.name).toBe('ClaiError')
  })

  it('should default to exit code 1', () => {
    const error = new ClaiError('test')
    expect(error.code).toBe(1)
  })

  it('should support error cause', () => {
    const cause = new Error('cause')
    const error = new ClaiError('wrapper', 1, cause)
    expect(error.cause).toBe(cause)
  })

  it('should support instanceof checks', () => {
    const error = new ClaiError('test', 1)
    expect(error instanceof ClaiError).toBe(true)
    expect(error instanceof Error).toBe(true)
  })

  it('should have readonly exit code property', () => {
    const error = new ClaiError('test', 42)
    // TypeScript enforces readonly at compile time
    // Runtime assignment is technically possible but should fail TS compilation
    expect(error.code).toBe(42)
    const descriptor = Object.getOwnPropertyDescriptor(error, 'code')
    expect(descriptor?.writable).toBe(false)
  })
})

describe('UsageError', () => {
  it('should have exit code 2', () => {
    const error = new UsageError('bad arg')
    expect(error.code).toBe(2)
    expect(error.name).toBe('UsageError')
    expect(error.message).toBe('bad arg')
  })

  it('should extend ClaiError', () => {
    const error = new UsageError('bad arg')
    expect(error instanceof ClaiError).toBe(true)
    expect(error instanceof Error).toBe(true)
  })

  it('should support error cause', () => {
    const cause = new Error('invalid')
    const error = new UsageError('bad arg', cause)
    expect(error.cause).toBe(cause)
  })
})

describe('InterruptError', () => {
  it('should have exit code 130', () => {
    const error = new InterruptError()
    expect(error.code).toBe(130)
    expect(error.name).toBe('InterruptError')
    expect(error.message).toBe('Interrupted')
  })

  it('should accept custom message', () => {
    const error = new InterruptError('SIGTERM received')
    expect(error.message).toBe('SIGTERM received')
    expect(error.code).toBe(130)
  })

  it('should extend ClaiError', () => {
    const error = new InterruptError()
    expect(error instanceof ClaiError).toBe(true)
    expect(error instanceof Error).toBe(true)
  })

  it('should support error cause', () => {
    const cause = new Error('signal')
    const error = new InterruptError('interrupted', cause)
    expect(error.cause).toBe(cause)
  })
})

describe('ConfigError', () => {
  it('should have default exit code 3', () => {
    const error = new ConfigError('config fail')
    expect(error.code).toBe(3)
    expect(error.name).toBe('ConfigError')
    expect(error.message).toBe('config fail')
  })

  it('should accept custom exit code', () => {
    const error = new ConfigError('config fail', 42)
    expect(error.code).toBe(42)
  })

  it('should extend ClaiError', () => {
    const error = new ConfigError('config fail', 3)
    expect(error instanceof ClaiError).toBe(true)
    expect(error instanceof Error).toBe(true)
  })
})

describe('ContextError', () => {
  it('should have default exit code 1', () => {
    const error = new ContextError('context fail')
    expect(error.code).toBe(1)
    expect(error.name).toBe('ContextError')
    expect(error.message).toBe('context fail')
  })

  it('should accept custom exit code', () => {
    const error = new ContextError('context fail', 42)
    expect(error.code).toBe(42)
  })

  it('should extend ClaiError', () => {
    const error = new ContextError('context fail', 1)
    expect(error instanceof ClaiError).toBe(true)
    expect(error instanceof Error).toBe(true)
  })
})

describe('AIError', () => {
  it('should have exit code 4', () => {
    const error = new AIError('api fail')
    expect(error.code).toBe(4)
    expect(error.name).toBe('AIError')
    expect(error.message).toBe('api fail')
  })

  it('should store HTTP status code', () => {
    const error = new AIError('rate limited', 429)
    expect(error.statusCode).toBe(429)
    expect(error.code).toBe(4)
  })

  it('should support error cause', () => {
    const cause = new Error('network')
    const error = new AIError('api fail', undefined, cause)
    expect(error.cause).toBe(cause)
  })

  it('should extend ClaiError', () => {
    const error = new AIError('api fail', 429)
    expect(error instanceof ClaiError).toBe(true)
    expect(error instanceof Error).toBe(true)
  })
})

describe('SafetyError', () => {
  it('should have exit code 5', () => {
    const error = new SafetyError('aborted')
    expect(error.code).toBe(5)
    expect(error.name).toBe('SafetyError')
    expect(error.message).toBe('aborted')
  })

  it('should support error cause', () => {
    const cause = new Error('timeout')
    const error = new SafetyError('aborted', cause)
    expect(error.cause).toBe(cause)
  })

  it('should extend ClaiError', () => {
    const error = new SafetyError('aborted')
    expect(error instanceof ClaiError).toBe(true)
    expect(error instanceof Error).toBe(true)
  })
})

describe('ExecutionError', () => {
  it('should have custom exit code', () => {
    const error = new ExecutionError('exec fail', 127)
    expect(error.code).toBe(127)
    expect(error.name).toBe('ExecutionError')
    expect(error.message).toBe('exec fail')
  })

  it('should support error cause', () => {
    const cause = new Error('spawn failed')
    const error = new ExecutionError('exec fail', 127, cause)
    expect(error.cause).toBe(cause)
  })

  it('should extend ClaiError', () => {
    const error = new ExecutionError('exec fail', 127)
    expect(error instanceof ClaiError).toBe(true)
    expect(error instanceof Error).toBe(true)
  })
})

describe('Error hierarchy', () => {
  it('should support instanceof checks across hierarchy', () => {
    const errors = [
      new UsageError('usage'),
      new InterruptError(),
      new ConfigError('config'),
      new ContextError('context'),
      new AIError('ai'),
      new SafetyError('safety'),
      new ExecutionError('exec', 1),
    ]

    errors.forEach((error) => {
      expect(error instanceof ClaiError).toBe(true)
      expect(error instanceof Error).toBe(true)
    })
  })

  it('should have correct exit codes per PRD', () => {
    expect(new UsageError('x').code).toBe(2)
    expect(new ConfigError('x').code).toBe(3)
    expect(new AIError('x').code).toBe(4)
    expect(new SafetyError('x').code).toBe(5)
    expect(new InterruptError().code).toBe(130)
    expect(new ContextError('x').code).toBe(1)
    expect(new ExecutionError('x', 127).code).toBe(127)
  })

  it('should preserve proper prototype chain', () => {
    const error = new ConfigError('test')
    expect(Object.getPrototypeOf(error)).toBe(ConfigError.prototype)
    expect(Object.getPrototypeOf(Object.getPrototypeOf(error))).toBe(
      ClaiError.prototype
    )
    expect(
      Object.getPrototypeOf(Object.getPrototypeOf(Object.getPrototypeOf(error)))
    ).toBe(Error.prototype)
  })

  it('should support error cause chaining', () => {
    const rootCause = new Error('root')
    const contextError = new ContextError('context failed', 1)
    const wrapperError = new ClaiError('wrapper', 1, contextError)

    expect(wrapperError.cause).toBe(contextError)

    const aiError = new AIError('api failed', 500, rootCause)
    expect(aiError.cause).toBe(rootCause)
  })
})
