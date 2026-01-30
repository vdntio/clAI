// Prompt builder for AI requests

import { ContextData } from '../context/types.js'
import { ChatMessage } from './types.js'

/**
 * Build chat messages for AI request
 * Constructs system and user messages based on context and instruction
 *
 * @param context - Gathered system/directory/history/stdin context
 * @param instruction - User's natural language instruction
 * @param numOptions - Number of command options to generate (1 for single, >1 for multi)
 * @returns Array of chat messages for the AI
 */
export function buildPrompt(
  context: ContextData,
  instruction: string,
  numOptions: number
): ChatMessage[] {
  const isMultiCommand = numOptions > 1

  // System message differs for single vs multi-command
  const systemMessage = isMultiCommand
    ? buildMultiCommandSystemPrompt(numOptions)
    : buildSingleCommandSystemPrompt()

  // User message includes context and instruction
  const userMessage = buildUserPrompt(context, instruction, numOptions)

  return [
    { role: 'system', content: systemMessage },
    { role: 'user', content: userMessage },
  ]
}

/**
 * Build system prompt for single command generation
 */
function buildSingleCommandSystemPrompt(): string {
  return `You are a helpful assistant that converts natural language instructions into executable shell commands. Respond with ONLY the command, no explanations or markdown.`
}

/**
 * Build system prompt for multi-command generation
 */
function buildMultiCommandSystemPrompt(numOptions: number): string {
  return `You are a helpful assistant that converts natural language instructions into executable shell commands. Generate exactly ${numOptions} different command options. Respond ONLY with a JSON object in this format: {"commands": ["cmd1", "cmd2", ...]}. No markdown, no explanations.`
}

/**
 * Build user prompt with context data
 */
function buildUserPrompt(
  context: ContextData,
  instruction: string,
  numOptions: number
): string {
  const parts: string[] = []

  // System context
  parts.push(`System Context:
OS: ${context.system.osName} ${context.system.osVersion}
Architecture: ${context.system.architecture}
Shell: ${context.system.shell}
User: ${context.system.user}
Memory: ${context.system.totalMemoryMb} MB`)

  // Directory context
  const filesList =
    context.files.length > 0
      ? context.files.slice(0, 20).join(', ')
      : '(empty directory)'
  parts.push(`\nDirectory Context:
Current directory: ${context.cwd}
Files: ${filesList}`)

  // History context (if available)
  if (context.history.length > 0) {
    const historyList = context.history
      .map((h, i) => `${i + 1}. ${h}`)
      .join('\n')
    parts.push(`\nRecent Shell History:\n${historyList}`)
  }

  // Stdin context (if available)
  if (context.stdin) {
    parts.push(`\nStdin input:\n${context.stdin}`)
  }

  // User instruction
  parts.push(`\nUser Instruction: ${instruction}`)

  // Response instruction differs for single vs multi
  if (numOptions > 1) {
    parts.push(
      `\nRespond with exactly ${numOptions} different command options as JSON: {"commands": ["cmd1", "cmd2", ...]}. Order from simplest to most advanced. No markdown or explanations.`
    )
  } else {
    parts.push(
      `\nRespond ONLY with the executable command. Do not include markdown code fences, explanations, or any other text. Just the command itself.`
    )
  }

  return parts.join('')
}
