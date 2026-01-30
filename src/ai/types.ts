// AI Types and Interfaces for the clai CLI
import { ClaiError } from '../error/index.js'

/**
 * Chat message for OpenAI-compatible API
 */
export interface ChatMessage {
  role: 'system' | 'user' | 'assistant'
  content: string
}

/**
 * Request to send to AI provider
 */
export interface ChatRequest {
  model: string
  messages: ChatMessage[]
  temperature?: number
  maxTokens?: number
}

/**
 * Response from AI provider
 */
export interface ChatResponse {
  content: string
  model?: string
  usage?: {
    promptTokens: number
    completionTokens: number
    totalTokens: number
  }
}

/**
 * Provider interface for future extensibility
 * Allows adding other providers (Anthropic, Ollama, etc.) in the future
 */
export interface AIProvider {
  name: string
  isAvailable(): boolean
  complete(request: ChatRequest): Promise<ChatResponse>
}

/**
 * Error class for AI operations
 * Exit code 4 as per PRD (API error)
 */
export class AIError extends ClaiError {
  public readonly statusCode?: number

  constructor(message: string, statusCode?: number, cause?: Error) {
    super(message, 4, cause)
    this.name = 'AIError'
    this.statusCode = statusCode
    Object.setPrototypeOf(this, AIError.prototype)
  }
}
