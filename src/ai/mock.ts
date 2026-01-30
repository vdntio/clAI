// Mock AI Provider for testing
// Returns simple echo commands without making real API calls
// Activated by setting MOCK_AI=1 environment variable

import { AIProvider, ChatRequest, ChatResponse } from './types.js'

/**
 * Mock provider for testing
 * Returns predictable responses without API calls
 */
export class MockProvider implements AIProvider {
  name = 'mock'

  /**
   * Always available (no API key needed)
   */
  isAvailable(): boolean {
    return true
  }

  /**
   * Return mock response based on request
   * Detects multi-command mode from system message
   */
  async complete(request: ChatRequest): Promise<ChatResponse> {
    const systemMsg = request.messages[0]?.content || ''
    const isMultiCommand = systemMsg.includes('JSON')

    if (isMultiCommand) {
      // Extract number of commands from system message
      const match = systemMsg.match(/exactly (\d+) different/)
      const numCommands = match?.[1] ? parseInt(match[1], 10) : 3

      const commands = Array.from(
        { length: numCommands },
        (_, i) => `echo "mock command ${i + 1}"`
      )

      return {
        content: JSON.stringify({ commands }),
        model: 'mock',
        usage: {
          promptTokens: 100,
          completionTokens: 50,
          totalTokens: 150,
        },
      }
    }

    // Single command
    return {
      content: 'echo "mock command"',
      model: 'mock',
      usage: {
        promptTokens: 50,
        completionTokens: 10,
        totalTokens: 60,
      },
    }
  }
}
