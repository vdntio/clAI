// Response parser for AI output

import { AIError } from './types.js'

/**
 * Parse AI response content into command(s)
 *
 * Handles:
 * - Markdown code fence stripping (```bash, ```sh, ```shell, ```json, ```)
 * - JSON parsing for multi-command mode: {"commands": [...]} or [...]
 * - Fallback to single trimmed command
 *
 * @param content - Raw response content from AI
 * @param expectMultiple - Whether to expect multiple commands (numOptions > 1)
 * @returns Array of command strings
 * @throws AIError if response is empty or cannot be parsed
 */
export function parseResponse(
  content: string,
  expectMultiple: boolean
): string[] {
  // 1. Trim whitespace
  let cleaned = content.trim()

  if (!cleaned) {
    throw new AIError('AI returned empty response')
  }

  // 2. Strip markdown code fences
  cleaned = stripCodeFences(cleaned)

  // Check if empty after stripping fences
  if (!cleaned) {
    throw new AIError('AI returned empty response after parsing')
  }

  // 3. Try JSON parse for multi-command mode
  if (expectMultiple) {
    const commands = tryParseMultipleCommands(cleaned)
    if (commands.length > 0) {
      return commands
    }
  }

  // 4. Fallback: treat as single command
  cleaned = cleaned.trim()
  if (!cleaned) {
    throw new AIError('AI returned empty response after parsing')
  }

  return [cleaned]
}

/**
 * Strip markdown code fences from content
 * Handles multiple fence formats
 */
function stripCodeFences(content: string): string {
  // Pattern 1: ```language\ncontent\n```
  // Pattern 2: ```\ncontent\n```
  // Must match the entire string from start to end

  const lines = content.split('\n')

  // Check if first line is an opening fence
  if (lines[0]?.startsWith('```')) {
    // Check if last line is a closing fence
    if (lines[lines.length - 1] === '```') {
      // Remove first and last lines
      return lines.slice(1, -1).join('\n').trim()
    }

    // Single line case: ```content```
    const singleLineMatch = content.match(/^```[\w]*\s*(.+?)\s*```$/)
    if (singleLineMatch?.[1]) {
      return singleLineMatch[1].trim()
    }
  }

  return content
}

/**
 * Try to parse multiple commands from JSON
 * Returns empty array if parsing fails
 */
function tryParseMultipleCommands(content: string): string[] {
  // Try to extract JSON object if content has extra text
  let jsonContent = content

  // Look for JSON object pattern if content has extra text
  const jsonMatch = content.match(/\{[\s\S]*"commands"[\s\S]*\}/)
  if (jsonMatch) {
    jsonContent = jsonMatch[0]
  }

  try {
    const parsed = JSON.parse(jsonContent)

    // Handle {"commands": [...]}
    if (parsed.commands && Array.isArray(parsed.commands)) {
      const cmds = parsed.commands.filter(
        (c: unknown) => typeof c === 'string' && c.trim()
      )
      return cmds.map((c: string) => c.trim())
    }

    // Handle raw array [...]
    if (Array.isArray(parsed)) {
      const cmds = parsed.filter(
        (c: unknown) => typeof c === 'string' && c.trim()
      )
      return cmds.map((c: string) => c.trim())
    }
  } catch {
    // JSON parse failed, try to extract from text
    // Look for array pattern: ["cmd1", "cmd2"]
    const arrayMatch = content.match(/\[[\s\S]*\]/)
    if (arrayMatch) {
      try {
        const parsed = JSON.parse(arrayMatch[0])
        if (Array.isArray(parsed)) {
          const cmds = parsed.filter(
            (c: unknown) => typeof c === 'string' && c.trim()
          )
          return cmds.map((c: string) => c.trim())
        }
      } catch {
        // JSON parse failed, continue to other strategies
      }
    }
  }

  return []
}
