import { readFileSync, accessSync, constants, statSync } from 'fs'
import { homedir } from 'os'
import { resolve, join } from 'path'
import TOML from '@iarna/toml'
import {
  FileConfig,
  FileConfigSchema,
  Config,
  ProviderConfig,
} from './types.js'
import { Cli } from '../cli/index.js'

// Config cache to avoid reloading
let configCache: FileConfig | null = null

// Default config values
const DEFAULT_CONFIG: FileConfig = {
  provider: {
    default: 'openrouter',
    fallback: [],
  },
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
  },
  providers: {},
}

// Config file paths in order of precedence (lowest to highest)
function getConfigPaths(): string[] {
  const paths: string[] = []

  // 1. /etc/clai/config.toml (lowest priority)
  paths.push('/etc/clai/config.toml')

  // 2. ~/.config/clai/config.toml
  const home = homedir()
  if (home) {
    paths.push(join(home, '.config', 'clai', 'config.toml'))
  }

  // 3. $XDG_CONFIG_HOME/clai/config.toml
  const xdgConfig = process.env.XDG_CONFIG_HOME
  if (xdgConfig) {
    paths.push(join(xdgConfig, 'clai', 'config.toml'))
  }

  // 4. ./.clai.toml (highest priority)
  paths.push(resolve('.clai.toml'))

  return paths
}

// Check if file has correct permissions (Unix 0600)
function checkFilePermissions(path: string): void {
  // Only check on Unix-like systems
  if (process.platform === 'win32') {
    return
  }

  try {
    const stats = statSync(path)
    const mode = stats.mode

    // Check if permissions are 0600 (owner read/write only)
    const expectedMode = 0o600
    const actualMode = mode & 0o777

    if (actualMode !== expectedMode) {
      throw new ConfigError(
        `Config file ${path} has insecure permissions ${actualMode.toString(8)}. Must be 0600.`,
        3
      )
    }
  } catch (err) {
    if (err instanceof ConfigError) {
      throw err
    }
    // If we can't stat the file, that's ok - it might not exist
  }
}

// Custom error class with exit code
export class ConfigError extends Error {
  code: number

  constructor(message: string, code: number = 1) {
    super(message)
    this.name = 'ConfigError'
    this.code = code
  }
}

// Check if file exists and is readable
function fileExists(path: string): boolean {
  try {
    accessSync(path, constants.R_OK)
    return true
  } catch {
    return false
  }
}

// Load a single config file
function loadConfigFile(path: string): Partial<FileConfig> {
  if (!fileExists(path)) {
    return {}
  }

  // Check permissions on Unix
  checkFilePermissions(path)

  try {
    const content = readFileSync(path, 'utf-8')
    const parsed = TOML.parse(content)

    // Transform kebab-case to camelCase for compatibility
    const transformed = transformConfig(parsed)

    // Validate with Zod
    const result = FileConfigSchema.safeParse(transformed)
    if (!result.success) {
      throw new ConfigError(
        `Invalid config file ${path}: ${result.error.message}`,
        3
      )
    }

    return result.data
  } catch (err) {
    if (err instanceof ConfigError) {
      throw err
    }
    if (err instanceof Error) {
      throw new ConfigError(
        `Failed to parse config file ${path}: ${err.message}`,
        3
      )
    }
    throw err
  }
}

// Transform config keys from kebab-case to camelCase
function transformConfig(obj: unknown): unknown {
  if (Array.isArray(obj)) {
    return obj.map(transformConfig)
  }

  if (obj && typeof obj === 'object') {
    const result: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(obj)) {
      // Convert kebab-case to camelCase
      const camelKey = key.replace(/-([a-z])/g, (_, letter) =>
        letter.toUpperCase()
      )
      result[camelKey] = transformConfig(value)
    }
    return result
  }

  return obj
}

// Deep merge two objects
function deepMerge<T extends Record<string, unknown>>(
  target: T,
  source: Partial<T>
): T {
  const result = { ...target }

  for (const key in source) {
    if (source[key] !== undefined) {
      if (
        typeof source[key] === 'object' &&
        !Array.isArray(source[key]) &&
        source[key] !== null &&
        typeof result[key] === 'object' &&
        !Array.isArray(result[key]) &&
        result[key] !== null
      ) {
        result[key] = deepMerge(
          result[key] as Record<string, unknown>,
          source[key] as Record<string, unknown>
        ) as T[Extract<keyof T, string>]
      } else {
        result[key] = source[key] as T[Extract<keyof T, string>]
      }
    }
  }

  return result
}

// Load environment variables that override config
function loadEnvConfig(): Partial<FileConfig> {
  const envConfig: Partial<FileConfig> = {}

  // Provider settings
  if (process.env.CLAI_PROVIDER_DEFAULT || process.env.CLAI_PROVIDER_FALLBACK) {
    envConfig.provider = {
      default:
        process.env.CLAI_PROVIDER_DEFAULT ?? DEFAULT_CONFIG.provider!.default,
      fallback: process.env.CLAI_PROVIDER_FALLBACK
        ? process.env.CLAI_PROVIDER_FALLBACK.split(',').map((s) => s.trim())
        : DEFAULT_CONFIG.provider!.fallback,
    }
  }

  // Context settings
  if (
    process.env.CLAI_CONTEXT_MAX_FILES ||
    process.env.CLAI_CONTEXT_MAX_HISTORY ||
    process.env.CLAI_CONTEXT_REDACT_PATHS ||
    process.env.CLAI_CONTEXT_REDACT_USERNAME
  ) {
    envConfig.context = {
      maxFiles: process.env.CLAI_CONTEXT_MAX_FILES
        ? parseInt(process.env.CLAI_CONTEXT_MAX_FILES, 10) ||
          DEFAULT_CONFIG.context!.maxFiles
        : DEFAULT_CONFIG.context!.maxFiles,
      maxHistory: process.env.CLAI_CONTEXT_MAX_HISTORY
        ? parseInt(process.env.CLAI_CONTEXT_MAX_HISTORY, 10) ||
          DEFAULT_CONFIG.context!.maxHistory
        : DEFAULT_CONFIG.context!.maxHistory,
      redactPaths: process.env.CLAI_CONTEXT_REDACT_PATHS
        ? process.env.CLAI_CONTEXT_REDACT_PATHS === 'true'
        : DEFAULT_CONFIG.context!.redactPaths,
      redactUsername: process.env.CLAI_CONTEXT_REDACT_USERNAME
        ? process.env.CLAI_CONTEXT_REDACT_USERNAME === 'true'
        : DEFAULT_CONFIG.context!.redactUsername,
    }
  }

  // Safety settings
  if (
    process.env.CLAI_SAFETY_CONFIRM_DANGEROUS ||
    process.env.CLAI_SAFETY_DANGEROUS_PATTERNS
  ) {
    envConfig.safety = {
      confirmDangerous: process.env.CLAI_SAFETY_CONFIRM_DANGEROUS
        ? process.env.CLAI_SAFETY_CONFIRM_DANGEROUS !== 'false'
        : DEFAULT_CONFIG.safety!.confirmDangerous,
      dangerousPatterns: process.env.CLAI_SAFETY_DANGEROUS_PATTERNS
        ? process.env.CLAI_SAFETY_DANGEROUS_PATTERNS.split(',').map((s) =>
            s.trim()
          )
        : DEFAULT_CONFIG.safety!.dangerousPatterns,
    }
  }

  // UI settings
  if (process.env.CLAI_UI_COLOR) {
    const color = process.env.CLAI_UI_COLOR
    if (color === 'auto' || color === 'always' || color === 'never') {
      envConfig.ui = {
        color,
        interactive: DEFAULT_CONFIG.ui!.interactive,
        debugLogFile: DEFAULT_CONFIG.ui!.debugLogFile,
      }
    }
  }

  return envConfig
}

// Load file config (with caching)
export function loadFileConfig(): FileConfig {
  if (configCache) {
    return configCache
  }

  // Start with defaults
  let config: FileConfig = { ...DEFAULT_CONFIG }

  // Load config files in order (lowest to highest priority)
  const configPaths = getConfigPaths()

  for (const path of configPaths) {
    try {
      const fileConfig = loadConfigFile(path)
      if (Object.keys(fileConfig).length > 0) {
        config = deepMerge(config, fileConfig)
      }
    } catch (err) {
      if (err instanceof ConfigError && err.code === 3) {
        throw err
      }
      // Non-fatal errors for missing files
    }
  }

  // Apply environment overrides
  const envConfig = loadEnvConfig()
  config = deepMerge(config, envConfig)

  // Cache the result
  configCache = config

  return config
}

// Clear config cache (useful for testing)
export function clearConfigCache(): void {
  configCache = null
}

// Build runtime config from file config + CLI
export function buildConfig(fileConfig: FileConfig, cli: Cli): Config {
  // Determine effective color mode
  let color: 'auto' | 'always' | 'never' =
    fileConfig.ui?.color ?? DEFAULT_CONFIG.ui!.color
  if (cli.noColor) {
    color = 'never'
  } else if (cli.color !== 'auto') {
    color = cli.color
  }

  // Interactive mode: file can enable, CLI can only add
  const interactive =
    (fileConfig.ui?.interactive ?? DEFAULT_CONFIG.ui!.interactive) ||
    cli.interactive

  // Clamp numOptions to 1-10
  const numOptions = Math.max(1, Math.min(10, cli.numOptions))

  // Resolve debug log file path
  let debugLogFile: string | undefined
  if (cli.debugFile !== undefined) {
    // CLI --debug-file (empty string means use default)
    if (cli.debugFile === '') {
      debugLogFile = join(homedir(), '.cache', 'clai', 'debug.log')
    } else {
      debugLogFile = resolve(cli.debugFile)
    }
  } else if (fileConfig.ui?.debugLogFile) {
    // From file config
    const home = homedir()
    debugLogFile = fileConfig.ui.debugLogFile
      .replace(/^~\//, home + '/')
      .replace(/^~$/, home)
  }

  return {
    provider: fileConfig.provider ?? DEFAULT_CONFIG.provider!,
    context: fileConfig.context ?? DEFAULT_CONFIG.context!,
    safety: fileConfig.safety ?? DEFAULT_CONFIG.safety!,
    ui: {
      color,
      debugLogFile,
      interactive,
      numOptions,
    },
    providers: fileConfig.providers ?? DEFAULT_CONFIG.providers!,

    // CLI overrides
    model: cli.model,
    providerName: cli.provider,
    quiet: cli.quiet,
    verbose: cli.verbose,
    force: cli.force,
    dryRun: cli.dryRun,
    contextFile: cli.context,
    offline: cli.offline,
    debug: cli.debug,
    debugFile: cli.debugFile,
    instruction: cli.instruction,
  }
}

// Main entry point: load and build complete config
export function getConfig(cli: Cli): Config {
  const fileConfig = loadFileConfig()
  return buildConfig(fileConfig, cli)
}

// Get API key for a provider (with env var resolution)
export function getProviderApiKey(
  providerName: string,
  config: Config
): string | undefined {
  const providerConfig = config.providers[providerName]

  if (!providerConfig) {
    // Fallback to OPENROUTER_API_KEY for openrouter
    if (providerName === 'openrouter') {
      return process.env.OPENROUTER_API_KEY
    }
    return undefined
  }

  // Priority: apiKey (with env var substitution) > apiKeyEnv > env var
  if (providerConfig.apiKey) {
    // Resolve env var references like ${VAR} or $VAR
    let apiKey = providerConfig.apiKey
    apiKey = apiKey.replace(
      /\$\{([^}]+)\}/g,
      (_, varName) => process.env[varName] || ''
    )
    apiKey = apiKey.replace(
      /\$([A-Za-z_][A-Za-z0-9_]*)/g,
      (_, varName) => process.env[varName] || ''
    )
    if (apiKey) return apiKey
  }

  if (providerConfig.apiKeyEnv) {
    return process.env[providerConfig.apiKeyEnv]
  }

  // Fallback for openrouter
  if (providerName === 'openrouter') {
    return process.env.OPENROUTER_API_KEY
  }

  return undefined
}

// Get model for a provider
export function getProviderModel(providerName: string, config: Config): string {
  // Priority: CLI --model > provider config > default
  if (config.model) {
    return config.model
  }

  const providerConfig = config.providers[providerName]
  if (providerConfig?.model) {
    return providerConfig.model
  }

  // Default models by provider
  if (providerName === 'openrouter') {
    return 'qwen/qwen3-coder'
  }

  return 'gpt-4o-mini'
}
