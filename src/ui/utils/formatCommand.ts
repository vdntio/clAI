// src/ui/utils/formatCommand.ts
// Utilities for formatting and truncating commands for display

/**
 * Format a command for display in the terminal
 * Truncates with ellipsis if too long for available width
 *
 * @param command - The command string to format
 * @param maxWidth - Maximum width available (in columns)
 * @returns Formatted command string
 */
export function formatCommand(command: string, maxWidth: number): string {
  // Account for "$ " prefix (2 chars) and some padding
  const availableWidth = maxWidth - 4

  if (availableWidth <= 3) {
    // Extremely narrow terminal
    return '...'
  }

  if (command.length <= availableWidth) {
    return command
  }

  // Truncate with ellipsis, preserving as much context as possible
  return command.slice(0, availableWidth - 3) + '...'
}

/**
 * Format command counter like [1/3]
 *
 * @param currentIndex - Current index (0-based)
 * @param total - Total number of commands
 * @returns Formatted counter string
 */
export function formatCounter(currentIndex: number, total: number): string {
  if (total <= 1) {
    return ''
  }
  return `[${currentIndex + 1}/${total}]`
}

/**
 * Calculate available width for command display
 * Accounts for prefix, counter, and padding
 *
 * @param terminalWidth - Total terminal width
 * @param hasCounter - Whether counter will be shown
 * @returns Width available for command text
 */
export function getCommandDisplayWidth(
  terminalWidth: number,
  hasCounter: boolean = true
): number {
  // Reserve space for:
  // - "$ " prefix (2 chars)
  // - Counter "[N/M]" (5-7 chars) if multiple commands
  // - Some padding (2-4 chars)
  const counterSpace = hasCounter ? 8 : 0
  const padding = 4
  return Math.max(20, terminalWidth - counterSpace - padding)
}

/**
 * Truncate text with ellipsis in the middle (for paths)
 *
 * @param text - Text to truncate
 * @param maxLength - Maximum length
 * @returns Truncated text
 */
export function truncateMiddle(text: string, maxLength: number): string {
  if (text.length <= maxLength) {
    return text
  }

  if (maxLength <= 5) {
    return text.slice(0, maxLength - 1) + '…'
  }

  const sideLength = Math.floor((maxLength - 3) / 2)
  const start = text.slice(0, sideLength)
  const end = text.slice(-sideLength)
  return `${start}...${end}`
}

/**
 * Wrap text to fit within a maximum width
 *
 * @param text - Text to wrap
 * @param maxWidth - Maximum line width
 * @returns Array of wrapped lines
 */
export function wrapText(text: string, maxWidth: number): string[] {
  if (text.length <= maxWidth) {
    return [text]
  }

  const words = text.split(/\s+/)
  const lines: string[] = []
  let currentLine = ''

  for (const word of words) {
    if (currentLine.length === 0) {
      currentLine = word
    } else if (currentLine.length + 1 + word.length <= maxWidth) {
      currentLine += ' ' + word
    } else {
      lines.push(currentLine)
      currentLine = word
    }
  }

  if (currentLine.length > 0) {
    lines.push(currentLine)
  }

  return lines
}

/**
 * Create a visual separator line
 *
 * @param width - Width of the separator
 * @param char - Character to use (default: ─)
 * @returns Separator string
 */
export function createSeparator(width: number, char: string = '─'): string {
  return char.repeat(Math.max(0, width))
}
