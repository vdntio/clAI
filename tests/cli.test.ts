import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { parseCli, type Cli } from '../src/cli/index.js'

describe('CLI Parser', () => {
  let exitSpy: ReturnType<typeof vi.spyOn>
  let stderrSpy: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    exitSpy = vi.spyOn(process, 'exit').mockImplementation((code) => {
      throw new Error(`process.exit(${code})`)
    })
    stderrSpy = vi.spyOn(process.stderr, 'write').mockImplementation(() => true)
  })

  afterEach(() => {
    exitSpy.mockRestore()
    stderrSpy.mockRestore()
  })

  function parse(args: string[]): Cli {
    return parseCli(['node', 'clai', ...args])
  }

  describe('instruction argument', () => {
    it('parses positional instruction', () => {
      const cli = parse(['find all rust files'])
      expect(cli.instruction).toBe('find all rust files')
    })

    it('throws UsageError with code 2 when instruction is missing', () => {
      const error = (() => {
        try {
          parse([])
          return null
        } catch (e) {
          return e
        }
      })()
      expect(error).toBeDefined()
      expect(error.name).toBe('UsageError')
      expect(error.code).toBe(2)
      expect(error.message).toContain('missing required argument')
    })
  })

  describe('model and provider options', () => {
    it('parses --model with -m shorthand', () => {
      const cli = parse(['-m', 'gpt-4', 'test instruction'])
      expect(cli.model).toBe('gpt-4')
    })

    it('parses --provider with -p shorthand', () => {
      const cli = parse(['-p', 'openai', 'test instruction'])
      expect(cli.provider).toBe('openai')
    })

    it('defaults to undefined when not provided', () => {
      const cli = parse(['test instruction'])
      expect(cli.model).toBeUndefined()
      expect(cli.provider).toBeUndefined()
    })
  })

  describe('verbosity options', () => {
    it('defaults quiet to false', () => {
      const cli = parse(['test'])
      expect(cli.quiet).toBe(false)
    })

    it('sets quiet with -q', () => {
      const cli = parse(['-q', 'test'])
      expect(cli.quiet).toBe(true)
    })

    it('defaults verbose to 0', () => {
      const cli = parse(['test'])
      expect(cli.verbose).toBe(0)
    })

    it('increments verbose with each -v', () => {
      expect(parse(['-v', 'test']).verbose).toBe(1)
      expect(parse(['-v', '-v', 'test']).verbose).toBe(2)
      expect(parse(['-vvv', 'test']).verbose).toBe(3)
    })
  })

  describe('color options', () => {
    it('defaults color to auto', () => {
      const cli = parse(['test'])
      expect(cli.color).toBe('auto')
    })

    it('parses --color always', () => {
      const cli = parse(['--color', 'always', 'test'])
      expect(cli.color).toBe('always')
    })

    it('parses --color never', () => {
      const cli = parse(['--color', 'never', 'test'])
      expect(cli.color).toBe('never')
    })
  })

  describe('mode flags', () => {
    it('defaults interactive to false', () => {
      const cli = parse(['test'])
      expect(cli.interactive).toBe(false)
    })

    it('sets interactive with -i', () => {
      const cli = parse(['-i', 'test'])
      expect(cli.interactive).toBe(true)
    })

    it('defaults force to false', () => {
      const cli = parse(['test'])
      expect(cli.force).toBe(false)
    })

    it('sets force with -f', () => {
      const cli = parse(['-f', 'test'])
      expect(cli.force).toBe(true)
    })

    it('defaults dryRun to false', () => {
      const cli = parse(['test'])
      expect(cli.dryRun).toBe(false)
    })

    it('sets dryRun with -n', () => {
      const cli = parse(['-n', 'test'])
      expect(cli.dryRun).toBe(true)
    })

    it('defaults offline to false', () => {
      const cli = parse(['test'])
      expect(cli.offline).toBe(false)
    })

    it('sets offline with --offline', () => {
      const cli = parse(['--offline', 'test'])
      expect(cli.offline).toBe(true)
    })
  })

  describe('options count', () => {
    it('defaults numOptions to 1', () => {
      const cli = parse(['test'])
      expect(cli.numOptions).toBe(1)
    })

    it('parses -o with value', () => {
      const cli = parse(['-o', '5', 'test'])
      expect(cli.numOptions).toBe(5)
    })

    it('clamps numOptions to minimum 1', () => {
      const cli = parse(['-o', '0', 'test'])
      expect(cli.numOptions).toBe(1)
    })

    it('clamps numOptions to maximum 10', () => {
      const cli = parse(['-o', '15', 'test'])
      expect(cli.numOptions).toBe(10)
    })
  })

  describe('context option', () => {
    it('defaults context to undefined', () => {
      const cli = parse(['test'])
      expect(cli.context).toBeUndefined()
    })

    it('parses -c with file path', () => {
      const cli = parse(['-c', '/path/to/context.txt', 'test'])
      expect(cli.context).toBe('/path/to/context.txt')
    })
  })

  describe('debug options', () => {
    it('defaults debug to false', () => {
      const cli = parse(['test'])
      expect(cli.debug).toBe(false)
    })

    it('sets debug with -d', () => {
      const cli = parse(['-d', 'test'])
      expect(cli.debug).toBe(true)
    })

    it('defaults debugFile to undefined', () => {
      const cli = parse(['test'])
      expect(cli.debugFile).toBeUndefined()
    })

    it('parses --debug-file with path', () => {
      const cli = parse(['--debug-file', '/path/to/debug.log', 'test'])
      expect(cli.debugFile).toBe('/path/to/debug.log')
    })
  })

  describe('combined flags', () => {
    it('parses multiple flags together', () => {
      const cli = parse([
        '-m',
        'gpt-4',
        '-p',
        'openrouter',
        '-i',
        '-f',
        '-n',
        '-o',
        '5',
        '-v',
        '-v',
        '-d',
        'find large files',
      ])

      expect(cli.instruction).toBe('find large files')
      expect(cli.model).toBe('gpt-4')
      expect(cli.provider).toBe('openrouter')
      expect(cli.interactive).toBe(true)
      expect(cli.force).toBe(true)
      expect(cli.dryRun).toBe(true)
      expect(cli.numOptions).toBe(5)
      expect(cli.verbose).toBe(2)
      expect(cli.debug).toBe(true)
    })
  })
})
