// OpenRouter API Provider Implementation
// Supports retry logic with exponential backoff for 429 rate limit errors

import { AIProvider, ChatRequest, ChatResponse, AIError } from '../types.js'

const OPENROUTER_URL = 'https://openrouter.ai/api/v1/chat/completions'
const TIMEOUT_MS = 60_000
const MAX_RETRIES = 3

/**
 * Sleep helper for retry delays
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/**
 * OpenRouter provider implementation
 * Handles API calls with authentication, timeout, and retry logic
 */
export class OpenRouterProvider implements AIProvider {
  name = 'openrouter'
  private apiKey: string

  constructor(apiKey: string) {
    this.apiKey = apiKey
  }

  /**
   * Check if provider is available (has API key)
   */
  isAvailable(): boolean {
    return !!this.apiKey && this.apiKey.length > 0
  }

  /**
   * Send completion request to OpenRouter
   * Retries on 429 rate limit with exponential backoff
   */
  async complete(request: ChatRequest): Promise<ChatResponse> {
    let lastError: Error | null = null

    for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
      try {
        const response = await this.makeRequest(request)

        if (response.ok) {
          const json = await response.json()
          return this.parseResponse(json)
        }

        // Handle specific status codes
        const body = await response.text()

        // 429: Rate limited - retry with backoff
        if (response.status === 429 && attempt < MAX_RETRIES - 1) {
          const delay = 1000 * Math.pow(2, attempt) // 1s, 2s, 4s
          await sleep(delay)
          continue
        }

        // Other errors: throw immediately
        throw this.mapError(response.status, body)
      } catch (err) {
        // If it's already an AIError, rethrow immediately
        if (err instanceof AIError) {
          throw err
        }

        // Network or other errors
        lastError = err as Error

        // Only retry network errors on first attempt
        if (attempt === 0 && lastError.message?.includes('fetch')) {
          continue
        }

        // Otherwise, throw on last attempt
        if (attempt === MAX_RETRIES - 1) {
          throw new AIError(
            `Network error: ${lastError.message || 'Unknown error'}`
          )
        }
      }
    }

    throw lastError || new AIError('Unknown error during API call')
  }

  /**
   * Make the actual HTTP request
   */
  private async makeRequest(request: ChatRequest): Promise<Response> {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), TIMEOUT_MS)

    try {
      return await fetch(OPENROUTER_URL, {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${this.apiKey}`,
          'Content-Type': 'application/json',
          'HTTP-Referer': 'https://github.com/clai',
          'X-Title': 'clai',
        },
        body: JSON.stringify({
          model: request.model,
          messages: request.messages,
          ...(request.temperature !== undefined && {
            temperature: request.temperature,
          }),
          ...(request.maxTokens !== undefined && {
            max_tokens: request.maxTokens,
          }),
        }),
        signal: controller.signal,
      })
    } finally {
      clearTimeout(timeoutId)
    }
  }

  /**
   * Map HTTP status to appropriate AIError
   */
  private mapError(status: number, body: string): AIError {
    switch (status) {
      case 401:
      case 403:
        return new AIError(
          `Authentication error (${status}): ${body || 'Invalid API key'}`,
          status
        )
      case 408:
      case 504:
        return new AIError(
          `Timeout error (${status}): ${body || 'Request timed out'}`,
          status
        )
      case 429:
        return new AIError(
          `Rate limit exceeded (${status}): ${body || 'Too many requests'}`,
          status
        )
      default:
        return new AIError(
          `API error (${status}): ${body || 'Unknown error'}`,
          status
        )
    }
  }

  /**
   * Parse successful response JSON
   */
  private parseResponse(json: unknown): ChatResponse {
    const response = json as {
      choices?: Array<{
        message?: {
          content?: string
        }
      }>
      model?: string
      usage?: {
        prompt_tokens?: number
        completion_tokens?: number
        total_tokens?: number
      }
    }

    const content = response.choices?.[0]?.message?.content

    if (!content) {
      throw new AIError('Invalid response: no content in choices')
    }

    return {
      content,
      model: response.model,
      usage: response.usage
        ? {
            promptTokens: response.usage.prompt_tokens ?? 0,
            completionTokens: response.usage.completion_tokens ?? 0,
            totalTokens: response.usage.total_tokens ?? 0,
          }
        : undefined,
    }
  }
}
