// src/ui/output.ts
// Pretty output formatting for non-interactive mode

const isTTY = process.stdout.isTTY

// ANSI color codes
const colors = {
  reset: '\x1b[0m',
  bold: '\x1b[1m',
  dim: '\x1b[2m',
  green: '\x1b[32m',
  cyan: '\x1b[36m',
  yellow: '\x1b[33m',
  red: '\x1b[31m',
}

function color(text: string, ...codes: string[]): string {
  if (!isTTY) return text
  return codes.join('') + text + colors.reset
}

/**
 * Print a command to stdout with nice formatting
 */
export function printCommand(command: string, isDangerous = false): void {
  if (isTTY) {
    const promptColor = isDangerous ? colors.red : colors.green
    const cmdColor = isDangerous ? colors.red : colors.cyan
    process.stdout.write(
      `${promptColor}${colors.bold}$${colors.reset} ${cmdColor}${command}${colors.reset}\n`
    )
  } else {
    // Clean output for piping
    process.stdout.write(command)
  }
}

/**
 * Print a warning to stderr
 */
export function printWarning(message: string): void {
  process.stderr.write(color(`⚠️  ${message}`, colors.yellow) + '\n')
}

/**
 * Print an error to stderr
 */
export function printError(message: string): void {
  process.stderr.write(color(`✗ ${message}`, colors.red) + '\n')
}

/**
 * Print a success message to stderr
 */
export function printSuccess(message: string): void {
  process.stderr.write(color(`✓ ${message}`, colors.green) + '\n')
}

/**
 * Print info to stderr
 */
export function printInfo(message: string): void {
  process.stderr.write(color(message, colors.dim) + '\n')
}
