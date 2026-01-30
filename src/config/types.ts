import { z } from 'zod'
import { Cli } from '../cli/index.js'

// Provider configuration
export const ProviderConfigSchema = z.object({
  apiKey: z.string().optional(),
  apiKeyEnv: z.string().optional(),
  model: z.string().optional(),
  endpoint: z.string().optional(),
})

export type ProviderConfig = z.infer<typeof ProviderConfigSchema>

// File config schema (from TOML files) - all nested objects are optional for partial configs
export const FileConfigSchema = z.object({
  provider: z
    .object({
      default: z.string().default('openrouter'),
      fallback: z.array(z.string()).default([]),
    })
    .optional(),
  context: z
    .object({
      maxFiles: z.number().int().min(1).max(100).default(10),
      maxHistory: z.number().int().min(0).max(50).default(3),
      redactPaths: z.boolean().default(false),
      redactUsername: z.boolean().default(false),
    })
    .optional(),
  safety: z
    .object({
      confirmDangerous: z.boolean().default(true),
      dangerousPatterns: z.array(z.string()).default([]),
    })
    .optional(),
  ui: z
    .object({
      color: z.enum(['auto', 'always', 'never']).default('auto'),
      debugLogFile: z.string().optional(),
      interactive: z.boolean().default(false),
      promptTimeout: z.number().int().min(0).max(300000).default(30000),
    })
    .optional(),
  providers: z.record(z.string(), ProviderConfigSchema).default({}),
})

export type FileConfig = z.infer<typeof FileConfigSchema>

// Runtime config (merged from file + env + CLI)
export interface Config {
  // Provider settings
  provider: {
    default: string
    fallback: string[]
  }

  // Context settings
  context: {
    maxFiles: number
    maxHistory: number
    redactPaths: boolean
    redactUsername: boolean
  }

  // Safety settings
  safety: {
    confirmDangerous: boolean
    dangerousPatterns: string[]
  }

  // UI settings
  ui: {
    color: 'auto' | 'always' | 'never'
    debugLogFile?: string
    interactive: boolean
    numOptions: number // 1-10, from CLI
    promptTimeout: number // milliseconds, 0 = no timeout, default 30000
  }

  // Provider-specific configs
  providers: Record<string, ProviderConfig>

  // CLI overrides
  model?: string
  providerName?: string
  quiet: boolean
  verbose: number
  force: boolean
  dryRun: boolean
  contextFile?: string
  offline: boolean
  debug: boolean
  debugFile?: string
  instruction: string
}
