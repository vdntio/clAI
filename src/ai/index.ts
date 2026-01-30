// AI module main entry point
// Provides generateCommands() for converting natural language to shell commands

import { ContextData } from '../context/types.js'
import { getProviderApiKey, getProviderModel } from '../config/index.js'
import type { Config } from '../config/types.js'
import {
  ChatMessage,
  ChatRequest,
  AIError,
  AIProvider,
} from './types.js'
import { buildPrompt } from './prompt.js'
import { parseResponse } from './parser.js'
import { OpenRouterProvider } from './providers/index.js'
import { MockProvider } from './mock.js'

// Re-export types for consumers
export {
  AIError,
} from './types.js'
export type {
  ChatMessage,
  ChatRequest,
  ChatResponse,
  AIProvider,
} from './types.js'
export { buildPrompt } from './prompt.js'
export { parseResponse } from './parser.js'
export { OpenRouterProvider } from './providers/index.js'
export { MockProvider } from './mock.js'

/**
 * Generate shell commands from natural language instruction
 *
 * @param context - Gathered context (system, directory, history, stdin)
 * @param instruction - User's natural language instruction
 * @param config - Runtime configuration
 * @returns Array of command strings (1 for single mode, N for multi mode)
 * @throws AIError on API failure or parse error (exit code 4)
 */
export async function generateCommands(
  context: ContextData,
  instruction: string,
  config: Config
): Promise<string[]> {
  const providerName = config.providerName || config.provider.default
  const numOptions = config.ui.numOptions

  // Get appropriate provider
  const provider = getProvider(providerName, config)

  // Build prompt messages
  const messages = buildPrompt(context, instruction, numOptions)

  // Get model (from CLI, config, or default)
  const model = getProviderModel(providerName, config)

  // Make API request
  const request: ChatRequest = {
    model,
    messages,
    temperature: 0.1, // Low temperature for more deterministic commands
  }

  const response = await provider.complete(request)

  // Parse response into command(s)
  return parseResponse(response.content, numOptions > 1)
}

/**
 * Get provider instance based on configuration
 * Returns mock provider if MOCK_AI=1 is set
 */
function getProvider(name: string, config: Config): AIProvider {
  // Check for mock mode
  if (process.env.MOCK_AI === '1') {
    return new MockProvider()
  }

  // Currently only OpenRouter is supported
  // Future: add more providers here (Anthropic, Ollama, etc.)
  if (name === 'openrouter') {
    const apiKey = getProviderApiKey('openrouter', config)

    if (!apiKey) {
      throw new AIError(
        'OpenRouter API key not configured. ' +
          'Set OPENROUTER_API_KEY environment variable or configure api_key in .clai.toml'
      )
    }

    return new OpenRouterProvider(apiKey)
  }

  throw new AIError(
    `Unknown provider: ${name}. ` + 'Currently only "openrouter" is supported.'
  )
}

/**
 * Format messages for debug output
 * Returns formatted string showing all messages
 */
export function formatPromptForDebug(messages: ChatMessage[]): string {
  return messages
    .map((msg) => {
      const role = msg.role.charAt(0).toUpperCase() + msg.role.slice(1)
      const content =
        msg.content.length > 500
          ? msg.content.substring(0, 500) + '...'
          : msg.content
      return `[${role}]\n${content}`
    })
    .join('\n\n')
}
