// AI module tests

import {
  describe,
  it,
  expect,
  vi,
  beforeEach,
  afterEach,
  type MockInstance,
} from 'vitest'
import {
  generateCommands,
  AIError,
  buildPrompt,
  parseResponse,
  formatPromptForDebug,
  OpenRouterProvider,
  MockProvider,
} from '../src/ai/index.js'
import { ContextData, SystemInfo } from '../src/context/types.js'
import { Config } from '../src/config/types.js'

describe('AI Module', () => {
  // Mock context for testing
  const mockSystemInfo: SystemInfo = {
    osName: 'Linux',
    osVersion: '5.15.0',
    architecture: 'x64',
    shell: 'bash',
    user: 'testuser',
    totalMemoryMb: 16384,
  }

  const mockContext: ContextData = {
    system: mockSystemInfo,
    cwd: '/home/testuser/projects',
    files: ['file1.txt', 'file2.js', 'README.md'],
    history: ['ls -la', 'git status'],
    stdin: undefined,
  }

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
      numOptions: 1,
    },
    providers: {},
    quiet: false,
    verbose: 0,
    force: false,
    dryRun: false,
    offline: false,
    debug: false,
    instruction: 'list all files',
  }

  describe('Prompt Building', () => {
    it('should build single command prompt', () => {
      const messages = buildPrompt(mockContext, 'list all files', 1)

      expect(messages).toHaveLength(2)
      expect(messages[0].role).toBe('system')
      expect(messages[0].content).toContain('Respond with ONLY the command')
      expect(messages[1].role).toBe('user')
      expect(messages[1].content).toContain('list all files')
      expect(messages[1].content).toContain('System Context:')
      expect(messages[1].content).toContain('Linux 5.15.0')
      expect(messages[1].content).toContain('Directory Context:')
      expect(messages[1].content).toContain('/home/testuser/projects')
      expect(messages[1].content).toContain('Recent Shell History:')
    })

    it('should build multi-command prompt', () => {
      const messages = buildPrompt(mockContext, 'list files', 3)

      expect(messages).toHaveLength(2)
      expect(messages[0].role).toBe('system')
      expect(messages[0].content).toContain('exactly 3 different')
      expect(messages[0].content).toContain('JSON')
      expect(messages[1].role).toBe('user')
      expect(messages[1].content).toContain('exactly 3 different')
      expect(messages[1].content).toContain('JSON')
    })

    it('should include stdin when available', () => {
      const contextWithStdin: ContextData = {
        ...mockContext,
        stdin: 'some input data',
      }

      const messages = buildPrompt(contextWithStdin, 'process input', 1)

      expect(messages[1].content).toContain('Stdin input:')
      expect(messages[1].content).toContain('some input data')
    })

    it('should not include history section when empty', () => {
      const contextNoHistory: ContextData = {
        ...mockContext,
        history: [],
      }

      const messages = buildPrompt(contextNoHistory, 'list files', 1)

      // Should not have Recent Shell History section
      expect(messages[1].content).not.toContain('Recent Shell History:')
    })

    it('should format prompt for debug output', () => {
      const messages = buildPrompt(mockContext, 'test', 1)
      const formatted = formatPromptForDebug(messages)

      expect(formatted).toContain('[System]')
      expect(formatted).toContain('[User]')
      expect(formatted).toContain('System Context:')
    })
  })

  describe('Response Parser', () => {
    it('should parse clean single command', () => {
      const result = parseResponse('ls -la', false)

      expect(result).toEqual(['ls -la'])
    })

    it('should parse command with bash code fence', () => {
      const result = parseResponse('```bash\nls -la\n```', false)

      expect(result).toEqual(['ls -la'])
    })

    it('should parse command with sh code fence', () => {
      const result = parseResponse('```sh\ncd /tmp\n```', false)

      expect(result).toEqual(['cd /tmp'])
    })

    it('should parse command with shell code fence', () => {
      const result = parseResponse('```shell\necho hello\n```', false)

      expect(result).toEqual(['echo hello'])
    })

    it('should parse command with generic code fence', () => {
      const result = parseResponse('```\npwd\n```', false)

      expect(result).toEqual(['pwd'])
    })

    it('should parse multi-command JSON object', () => {
      const content = JSON.stringify({
        commands: ['ls -la', 'ls -lah', 'ls -l'],
      })

      const result = parseResponse(content, true)

      expect(result).toEqual(['ls -la', 'ls -lah', 'ls -l'])
    })

    it('should parse multi-command JSON array', () => {
      const content = JSON.stringify(['cmd1', 'cmd2', 'cmd3'])

      const result = parseResponse(content, true)

      expect(result).toEqual(['cmd1', 'cmd2', 'cmd3'])
    })

    it('should parse JSON inside markdown fence', () => {
      const content = '```json\n{"commands": ["echo 1", "echo 2"]}\n```'

      const result = parseResponse(content, true)

      expect(result).toEqual(['echo 1', 'echo 2'])
    })

    it('should handle JSON embedded in text', () => {
      const content =
        'Here are the commands: {"commands": ["ls", "pwd"]} Thanks!'

      const result = parseResponse(content, true)

      expect(result).toEqual(['ls', 'pwd'])
    })

    it('should fallback to single command when JSON parse fails', () => {
      const content = 'invalid json {'

      const result = parseResponse(content, true)

      expect(result).toEqual(['invalid json {'])
    })

    it('should throw on empty response', () => {
      expect(() => parseResponse('', false)).toThrow(AIError)
      expect(() => parseResponse('   ', false)).toThrow(AIError)
    })

    it('should throw on empty response after stripping fences', () => {
      expect(() => parseResponse('```bash\n```', false)).toThrow(AIError)
    })

    it('should filter empty commands from JSON array', () => {
      const content = JSON.stringify({
        commands: ['cmd1', '', 'cmd2', '  ', 'cmd3'],
      })

      const result = parseResponse(content, true)

      expect(result).toEqual(['cmd1', 'cmd2', 'cmd3'])
    })
  })

  describe('OpenRouter Provider', () => {
    let provider: OpenRouterProvider

    beforeEach(() => {
      provider = new OpenRouterProvider('test-api-key')
    })

    it('should be available with API key', () => {
      expect(provider.isAvailable()).toBe(true)
    })

    it('should not be available without API key', () => {
      const noKeyProvider = new OpenRouterProvider('')
      expect(noKeyProvider.isAvailable()).toBe(false)
    })

    it('should make successful API call', async () => {
      const mockResponse = {
        choices: [
          {
            message: {
              content: 'ls -la',
            },
          },
        ],
        model: 'qwen/qwen3-coder',
        usage: {
          prompt_tokens: 100,
          completion_tokens: 10,
          total_tokens: 110,
        },
      }

      ;(global as any).fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      })

      const request = {
        model: 'qwen/qwen3-coder',
        messages: [{ role: 'system' as const, content: 'test' }],
      }

      const response = await provider.complete(request)

      expect(response.content).toBe('ls -la')
      expect(response.model).toBe('qwen/qwen3-coder')
      expect(response.usage).toEqual({
        promptTokens: 100,
        completionTokens: 10,
        totalTokens: 110,
      })
    })

    it('should retry on 429 with exponential backoff', async () => {
      const mockSuccess = {
        choices: [{ message: { content: 'success' } }],
        model: 'test',
      }

      ;(global as any).fetch = vi
        .fn()
        .mockResolvedValueOnce({
          ok: false,
          status: 429,
          text: () => Promise.resolve('Rate limited'),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: () => Promise.resolve(mockSuccess),
        })

      const request = {
        model: 'test',
        messages: [{ role: 'user' as const, content: 'test' }],
      }

      const response = await provider.complete(request)

      expect(response.content).toBe('success')
      expect(fetch).toHaveBeenCalledTimes(2)
    })

    it('should throw AIError on 401', async () => {
      ;(global as any).fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 401,
        text: () => Promise.resolve('Unauthorized'),
      })

      const request = {
        model: 'test',
        messages: [{ role: 'user' as const, content: 'test' }],
      }

      await expect(provider.complete(request)).rejects.toThrow(AIError)
      await expect(provider.complete(request)).rejects.toThrow(
        'Authentication error'
      )
    })

    it('should throw AIError on 403', async () => {
      ;(global as any).fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 403,
        text: () => Promise.resolve('Forbidden'),
      })

      const request = {
        model: 'test',
        messages: [{ role: 'user' as const, content: 'test' }],
      }

      await expect(provider.complete(request)).rejects.toThrow(AIError)
    })

    it('should throw AIError on 408', async () => {
      ;(global as any).fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 408,
        text: () => Promise.resolve('Timeout'),
      })

      const request = {
        model: 'test',
        messages: [{ role: 'user' as const, content: 'test' }],
      }

      await expect(provider.complete(request)).rejects.toThrow(AIError)
      await expect(provider.complete(request)).rejects.toThrow('Timeout error')
    })

    it('should throw AIError on 504', async () => {
      ;(global as any).fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 504,
        text: () => Promise.resolve('Gateway timeout'),
      })

      const request = {
        model: 'test',
        messages: [{ role: 'user' as const, content: 'test' }],
      }

      await expect(provider.complete(request)).rejects.toThrow(AIError)
      await expect(provider.complete(request)).rejects.toThrow('Timeout error')
    })

    it('should throw AIError after 3 failed 429 retries', async () => {
      const fetchMock = vi.fn().mockResolvedValue({
        ok: false,
        status: 429,
        text: () => Promise.resolve('Rate limited'),
      })
      ;(global as any).fetch = fetchMock

      const request = {
        model: 'test',
        messages: [{ role: 'user' as const, content: 'test' }],
      }

      await expect(provider.complete(request)).rejects.toThrow(
        /Rate limit exceeded/
      )
      expect(fetchMock).toHaveBeenCalledTimes(3)
    }, 10000) // 10 second timeout for retry delays (1s + 2s + 4s)

    it('should throw AIError on network error', async () => {
      ;(global as any).fetch = vi
        .fn()
        .mockRejectedValue(new Error('Network error'))

      const request = {
        model: 'test',
        messages: [{ role: 'user' as const, content: 'test' }],
      }

      await expect(provider.complete(request)).rejects.toThrow(AIError)
      await expect(provider.complete(request)).rejects.toThrow('Network error')
    })

    it('should throw AIError on invalid response', async () => {
      ;(global as any).fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ noContent: true }),
      })

      const request = {
        model: 'test',
        messages: [{ role: 'user' as const, content: 'test' }],
      }

      await expect(provider.complete(request)).rejects.toThrow(AIError)
      await expect(provider.complete(request)).rejects.toThrow(
        'Invalid response'
      )
    })
  })

  describe('Mock Provider', () => {
    let provider: MockProvider

    beforeEach(() => {
      provider = new MockProvider()
    })

    it('should always be available', () => {
      expect(provider.isAvailable()).toBe(true)
    })

    it('should return single mock command', async () => {
      const request = {
        model: 'mock',
        messages: [
          { role: 'system' as const, content: 'single command' },
          { role: 'user' as const, content: 'test' },
        ],
      }

      const response = await provider.complete(request)

      expect(response.content).toBe('echo "mock command"')
      expect(response.model).toBe('mock')
      expect(response.usage).toBeDefined()
    })

    it('should return multi mock commands', async () => {
      const request = {
        model: 'mock',
        messages: [
          {
            role: 'system' as const,
            content: 'exactly 3 different JSON commands',
          },
          { role: 'user' as const, content: 'test' },
        ],
      }

      const response = await provider.complete(request)

      expect(response.content).toContain('mock command 1')
      expect(response.content).toContain('mock command 2')
      expect(response.content).toContain('mock command 3')
      expect(response.model).toBe('mock')
    })
  })

  describe('Integration', () => {
    let originalEnv: string | undefined

    beforeEach(() => {
      originalEnv = process.env.MOCK_AI
    })

    afterEach(() => {
      if (originalEnv !== undefined) {
        process.env.MOCK_AI = originalEnv
      } else {
        delete process.env.MOCK_AI
      }
      vi.restoreAllMocks()
    })

    it('should use mock provider when MOCK_AI=1', async () => {
      process.env.MOCK_AI = '1'

      const commands = await generateCommands(
        mockContext,
        'list files',
        mockConfig
      )

      expect(commands).toEqual(['echo "mock command"'])
    })

    it('should throw AIError when API key is missing', async () => {
      // Ensure MOCK_AI is not set
      delete process.env.MOCK_AI

      // Config with no API key configured
      const noKeyConfig: Config = {
        ...mockConfig,
        providers: {},
      }

      await expect(
        generateCommands(mockContext, 'list files', noKeyConfig)
      ).rejects.toThrow(AIError)
    })

    it('should use provided API key', async () => {
      delete process.env.MOCK_AI

      const mockResponse = {
        choices: [{ message: { content: 'ls -la' } }],
        model: 'test',
      }

      ;(global as any).fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      })

      const configWithKey: Config = {
        ...mockConfig,
        providers: {
          openrouter: { apiKey: 'sk-test123' },
        },
      }

      const commands = await generateCommands(
        mockContext,
        'list files',
        configWithKey
      )

      expect(commands).toEqual(['ls -la'])
      expect(fetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            Authorization: 'Bearer sk-test123',
          }),
        })
      )
    })

    it('should generate multiple commands', async () => {
      process.env.MOCK_AI = '1'

      const multiConfig: Config = {
        ...mockConfig,
        ui: { ...mockConfig.ui, numOptions: 3 },
      }

      const commands = await generateCommands(
        mockContext,
        'list files',
        multiConfig
      )

      expect(commands).toHaveLength(3)
      expect(commands[0]).toContain('mock command 1')
      expect(commands[1]).toContain('mock command 2')
      expect(commands[2]).toContain('mock command 3')
    })
  })

  describe('AIError', () => {
    it('should have default code 4', () => {
      const error = new AIError('Test error')
      expect(error.code).toBe(4)
      expect(error.name).toBe('AIError')
    })

    it('should include status code when provided', () => {
      const error = new AIError('API error', 429)
      expect(error.code).toBe(4)
      expect(error.statusCode).toBe(429)
    })
  })
})
