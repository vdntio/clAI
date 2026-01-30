// src/ui/spinner.ts
// Simple terminal spinner for loading states (non-Ink)

const FRAMES = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
const INTERVAL = 80

export interface SpinnerInstance {
  stop: (finalMessage?: string) => void
  update: (message: string) => void
}

/**
 * Create a simple terminal spinner
 * Renders to stderr to keep stdout clean
 */
export function createSpinner(message: string): SpinnerInstance {
  // Only show spinner in TTY mode
  if (!process.stderr.isTTY) {
    return {
      stop: () => {},
      update: () => {},
    }
  }

  let frameIndex = 0
  let currentMessage = message
  let stopped = false

  const render = () => {
    if (stopped) return
    const frame = FRAMES[frameIndex % FRAMES.length]
    process.stderr.write(`\r\x1b[36m${frame}\x1b[0m ${currentMessage}`)
    frameIndex++
  }

  const timer = setInterval(render, INTERVAL)
  render()

  return {
    stop: (finalMessage?: string) => {
      stopped = true
      clearInterval(timer)
      // Clear the line
      process.stderr.write('\r\x1b[K')
      if (finalMessage) {
        process.stderr.write(`${finalMessage}\n`)
      }
    },
    update: (msg: string) => {
      currentMessage = msg
    },
  }
}

/**
 * Run an async function with a spinner
 */
export async function withSpinner<T>(
  message: string,
  fn: () => Promise<T>,
  successMessage?: string
): Promise<T> {
  const spinner = createSpinner(message)
  try {
    const result = await fn()
    spinner.stop(successMessage)
    return result
  } catch (error) {
    spinner.stop()
    throw error
  }
}
