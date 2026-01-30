import { describe, it, expect, beforeEach, afterEach } from 'vitest'
import {
  loadFileConfig,
  buildConfig,
  clearConfigCache,
  ConfigError,
  getProviderApiKey,
  getProviderModel,
} from '../src/config/index.js'
import { FileConfig, Config } from '../src/config/types.js'
import { Cli } from '../src/cli/index.js'
import { mkdirSync, writeFileSync, rmSync, chmodSync } from 'fs'
import { join } from 'path'

describe('Config Module', () => {
  const testDir = join(process.cwd(), 'test-configs')

  beforeEach(() => {
    // Clear cache before each test
    clearConfigCache()

    // Create test directory
    try {
      mkdirSync(testDir, { recursive: true })
    } catch {
      // Directory may already exist
    }
  })

  afterEach(() => {
    // Clean up test files
    clearConfigCache()
    try {
      rmSync(testDir, { recursive: true, force: true })
    } catch {
      // Directory may not exist
    }
  })

  describe('FileConfig Loading', () => {
    it('should return defaults when no config files exist', () => {
      // Change to test directory to avoid loading project root's .clai.toml
      const originalCwd = process.cwd()
      process.chdir(testDir)

      try {
        const config = loadFileConfig()

        expect(config.provider.default).toBe('openrouter')
        expect(config.provider.fallback).toEqual([])
        expect(config.context.maxFiles).toBe(10)
        expect(config.context.maxHistory).toBe(3)
        expect(config.context.redactPaths).toBe(false)
        expect(config.safety.confirmDangerous).toBe(true)
        expect(config.ui.color).toBe('auto')
      } finally {
        process.chdir(originalCwd)
      }
    })

    it('should load and parse TOML config file', () => {
      const configContent = `
[provider]
default = "anthropic"
fallback = ["openrouter"]

[context]
max-files = 20
max-history = 5
redact-paths = true

[safety]
confirm-dangerous = false

[ui]
color = "always"
interactive = true
`
      writeFileSync(join(testDir, '.clai.toml'), configContent)
      chmodSync(join(testDir, '.clai.toml'), 0o600)

      // Temporarily change cwd
      const originalCwd = process.cwd()
      process.chdir(testDir)
      clearConfigCache()

      try {
        const config = loadFileConfig()

        expect(config.provider.default).toBe('anthropic')
        expect(config.provider.fallback).toEqual(['openrouter'])
        expect(config.context.maxFiles).toBe(20)
        expect(config.context.maxHistory).toBe(5)
        expect(config.context.redactPaths).toBe(true)
        expect(config.safety.confirmDangerous).toBe(false)
        expect(config.ui.color).toBe('always')
        expect(config.ui.interactive).toBe(true)
      } finally {
        process.chdir(originalCwd)
      }
    })

    it('should merge multiple config files by precedence', () => {
      // Lower priority file
      const etcConfig = `
[provider]
default = "ollama"

[context]
max-files = 50
`
      // Higher priority file
      const localConfig = `
[context]
max-files = 15
redact-paths = true
`

      writeFileSync(join(testDir, 'config.toml'), etcConfig)
      writeFileSync(join(testDir, '.clai.toml'), localConfig)
      chmodSync(join(testDir, '.clai.toml'), 0o600)

      const originalCwd = process.cwd()
      process.chdir(testDir)
      clearConfigCache()

      try {
        // Mock the /etc path (we can't actually write there)
        const config = loadFileConfig()

        // Should get local values
        expect(config.context.maxFiles).toBe(15)
        expect(config.context.redactPaths).toBe(true)
      } finally {
        process.chdir(originalCwd)
      }
    })

    it('should throw ConfigError with code 3 on parse error', () => {
      const invalidConfig = `
[context]
max-files = "not a number"
`
      writeFileSync(join(testDir, '.clai.toml'), invalidConfig)
      chmodSync(join(testDir, '.clai.toml'), 0o600)

      const originalCwd = process.cwd()
      process.chdir(testDir)
      clearConfigCache()

      try {
        expect(() => loadFileConfig()).toThrow(ConfigError)
        expect(() => loadFileConfig()).toThrow(/Invalid config file/)
      } finally {
        process.chdir(originalCwd)
      }
    })

    it('should check Unix permissions (0600) on config files', () => {
      // Skip on Windows
      if (process.platform === 'win32') {
        return
      }

      const configContent = `
[provider]
default = "test"
`
      const configPath = join(testDir, '.clai.toml')
      writeFileSync(configPath, configContent)
      // Set insecure permissions (0644)
      chmodSync(configPath, 0o644)

      const originalCwd = process.cwd()
      process.chdir(testDir)
      clearConfigCache()

      try {
        expect(() => loadFileConfig()).toThrow(ConfigError)
        expect(() => loadFileConfig()).toThrow(/insecure permissions/)
      } finally {
        process.chdir(originalCwd)
      }
    })

    it('should cache config after first load', () => {
      const configContent = `
[provider]
default = "cached"
`
      writeFileSync(join(testDir, '.clai.toml'), configContent)
      chmodSync(join(testDir, '.clai.toml'), 0o600)

      const originalCwd = process.cwd()
      process.chdir(testDir)
      clearConfigCache()

      try {
        const config1 = loadFileConfig()
        expect(config1.provider.default).toBe('cached')

        // Modify the file (should not affect cached config)
        writeFileSync(
          join(testDir, '.clai.toml'),
          '[provider]\ndefault = "modified"'
        )
        chmodSync(join(testDir, '.clai.toml'), 0o600)

        const config2 = loadFileConfig()
        expect(config2.provider.default).toBe('cached') // Still cached

        // Clear cache and reload
        clearConfigCache()
        const config3 = loadFileConfig()
        expect(config3.provider.default).toBe('modified')
      } finally {
        process.chdir(originalCwd)
      }
    })
  })

  describe('Environment Variable Override', () => {
    it('should override config with CLAI_* environment variables', () => {
      const originalEnv = { ...process.env }

      process.env.CLAI_PROVIDER_DEFAULT = 'test-provider'
      process.env.CLAI_CONTEXT_MAX_FILES = '25'
      process.env.CLAI_SAFETY_CONFIRM_DANGEROUS = 'false'
      process.env.CLAI_UI_COLOR = 'never'

      clearConfigCache()

      try {
        const config = loadFileConfig()

        expect(config.provider.default).toBe('test-provider')
        expect(config.context.maxFiles).toBe(25)
        expect(config.safety.confirmDangerous).toBe(false)
        expect(config.ui.color).toBe('never')
      } finally {
        process.env = originalEnv
      }
    })

    it('should handle CLAI_PROVIDER_FALLBACK as comma-separated list', () => {
      const originalEnv = { ...process.env }

      process.env.CLAI_PROVIDER_FALLBACK = 'provider1, provider2, provider3'
      clearConfigCache()

      try {
        const config = loadFileConfig()
        expect(config.provider.fallback).toEqual([
          'provider1',
          'provider2',
          'provider3',
        ])
      } finally {
        process.env = originalEnv
      }
    })

    it('should handle CLAI_SAFETY_DANGEROUS_PATTERNS as comma-separated list', () => {
      const originalEnv = { ...process.env }

      process.env.CLAI_SAFETY_DANGEROUS_PATTERNS = 'rm -rf, dd if=, mkfs'
      clearConfigCache()

      try {
        const config = loadFileConfig()
        expect(config.safety.dangerousPatterns).toEqual([
          'rm -rf',
          'dd if=',
          'mkfs',
        ])
      } finally {
        process.env = originalEnv
      }
    })
  })

  describe('Config Building', () => {
    it('should build runtime config from file config and CLI', () => {
      const fileConfig: FileConfig = {
        provider: { default: 'openrouter', fallback: [] },
        context: {
          maxFiles: 10,
          maxHistory: 3,
          redactPaths: false,
          redactUsername: false,
        },
        safety: { confirmDangerous: true, dangerousPatterns: [] },
        ui: { color: 'auto', interactive: false },
        providers: {},
      }

      const cli: Cli = {
        instruction: 'test command',
        model: 'gpt-4',
        provider: 'anthropic',
        quiet: true,
        verbose: 2,
        noColor: false,
        color: 'always',
        interactive: true,
        force: true,
        dryRun: true,
        offline: false,
        numOptions: 5,
        debug: true,
        debugFile: '/tmp/debug.log',
      }

      const config = buildConfig(fileConfig, cli)

      expect(config.instruction).toBe('test command')
      expect(config.model).toBe('gpt-4')
      expect(config.providerName).toBe('anthropic')
      expect(config.quiet).toBe(true)
      expect(config.verbose).toBe(2)
      expect(config.ui.color).toBe('always') // CLI overrides file
      expect(config.ui.interactive).toBe(true) // CLI or file
      expect(config.force).toBe(true)
      expect(config.dryRun).toBe(true)
      expect(config.ui.numOptions).toBe(5)
      expect(config.debug).toBe(true)
      expect(config.debugFile).toBe('/tmp/debug.log')
    })

    it('should clamp numOptions to 1-10 range', () => {
      const fileConfig: FileConfig = {
        provider: { default: 'openrouter', fallback: [] },
        context: {
          maxFiles: 10,
          maxHistory: 3,
          redactPaths: false,
          redactUsername: false,
        },
        safety: { confirmDangerous: true, dangerousPatterns: [] },
        ui: { color: 'auto', interactive: false },
        providers: {},
      }

      const cliLow: Cli = {
        instruction: 'test',
        numOptions: 0,
        quiet: false,
        verbose: 0,
        noColor: false,
        color: 'auto',
        interactive: false,
        force: false,
        dryRun: false,
        offline: false,
        debug: false,
      }

      const cliHigh: Cli = {
        instruction: 'test',
        numOptions: 15,
        quiet: false,
        verbose: 0,
        noColor: false,
        color: 'auto',
        interactive: false,
        force: false,
        dryRun: false,
        offline: false,
        debug: false,
      }

      const configLow = buildConfig(fileConfig, cliLow)
      const configHigh = buildConfig(fileConfig, cliHigh)

      expect(configLow.ui.numOptions).toBe(1)
      expect(configHigh.ui.numOptions).toBe(10)
    })

    it('should handle --no-color flag', () => {
      const fileConfig: FileConfig = {
        provider: { default: 'openrouter', fallback: [] },
        context: {
          maxFiles: 10,
          maxHistory: 3,
          redactPaths: false,
          redactUsername: false,
        },
        safety: { confirmDangerous: true, dangerousPatterns: [] },
        ui: { color: 'always', interactive: false },
        providers: {},
      }

      const cli: Cli = {
        instruction: 'test',
        numOptions: 3,
        quiet: false,
        verbose: 0,
        noColor: true,
        color: 'auto',
        interactive: false,
        force: false,
        dryRun: false,
        offline: false,
        debug: false,
      }

      const config = buildConfig(fileConfig, cli)
      expect(config.ui.color).toBe('never')
    })

    it('should handle empty --debug-file flag', () => {
      const fileConfig: FileConfig = {
        provider: { default: 'openrouter', fallback: [] },
        context: {
          maxFiles: 10,
          maxHistory: 3,
          redactPaths: false,
          redactUsername: false,
        },
        safety: { confirmDangerous: true, dangerousPatterns: [] },
        ui: { color: 'auto', interactive: false },
        providers: {},
      }

      const cli: Cli = {
        instruction: 'test',
        numOptions: 3,
        quiet: false,
        verbose: 0,
        noColor: false,
        color: 'auto',
        interactive: false,
        force: false,
        dryRun: false,
        offline: false,
        debug: false,
        debugFile: '', // Empty string means use default
      }

      const config = buildConfig(fileConfig, cli)
      expect(config.ui.debugLogFile).toContain('.cache/clai/debug.log')
    })
  })

  describe('Provider API Key Resolution', () => {
    it('should get API key from provider config', () => {
      const config = {
        providers: {
          openrouter: {
            apiKey: 'sk-test-key',
          },
        },
      } as unknown as Config

      const apiKey = getProviderApiKey('openrouter', config)
      expect(apiKey).toBe('sk-test-key')
    })

    it('should resolve env var references in apiKey', () => {
      const originalEnv = process.env.TEST_API_KEY
      process.env.TEST_API_KEY = 'resolved-key'

      const config = {
        providers: {
          openrouter: {
            apiKey: '${TEST_API_KEY}',
          },
        },
      } as unknown as Config

      try {
        const apiKey = getProviderApiKey('openrouter', config)
        expect(apiKey).toBe('resolved-key')
      } finally {
        if (originalEnv) {
          process.env.TEST_API_KEY = originalEnv
        } else {
          delete process.env.TEST_API_KEY
        }
      }
    })

    it('should fall back to apiKeyEnv', () => {
      const originalEnv = process.env.CUSTOM_KEY_VAR
      process.env.CUSTOM_KEY_VAR = 'custom-key-value'

      const config = {
        providers: {
          test: {
            apiKeyEnv: 'CUSTOM_KEY_VAR',
          },
        },
      } as unknown as Config

      try {
        const apiKey = getProviderApiKey('test', config)
        expect(apiKey).toBe('custom-key-value')
      } finally {
        if (originalEnv) {
          process.env.CUSTOM_KEY_VAR = originalEnv
        } else {
          delete process.env.CUSTOM_KEY_VAR
        }
      }
    })

    it('should fall back to OPENROUTER_API_KEY env var', () => {
      const originalEnv = process.env.OPENROUTER_API_KEY
      process.env.OPENROUTER_API_KEY = 'fallback-key'

      const config = {
        providers: {},
      } as unknown as Config

      try {
        const apiKey = getProviderApiKey('openrouter', config)
        expect(apiKey).toBe('fallback-key')
      } finally {
        if (originalEnv) {
          process.env.OPENROUTER_API_KEY = originalEnv
        } else {
          delete process.env.OPENROUTER_API_KEY
        }
      }
    })
  })

  describe('Provider Model Resolution', () => {
    it('should get model from CLI override', () => {
      const config = {
        model: 'cli-model',
        providers: {
          openrouter: { model: 'config-model' },
        },
      } as unknown as Config

      const model = getProviderModel('openrouter', config)
      expect(model).toBe('cli-model')
    })

    it('should get model from provider config', () => {
      const config = {
        providers: {
          openrouter: { model: 'config-model' },
        },
      } as unknown as Config

      const model = getProviderModel('openrouter', config)
      expect(model).toBe('config-model')
    })

    it('should return default model for openrouter', () => {
      const config = {
        providers: {},
      } as unknown as Config

      const model = getProviderModel('openrouter', config)
      expect(model).toBe('qwen/qwen3-coder')
    })

    it('should return generic default for unknown provider', () => {
      const config = {
        providers: {},
      } as unknown as Config

      const model = getProviderModel('unknown', config)
      expect(model).toBe('gpt-4o-mini')
    })
  })
})
