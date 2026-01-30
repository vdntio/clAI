import { InterruptError } from '../error/index.js'

// Interrupt flag set by signal handlers
let interrupted = false

// Register signal handlers
export function registerSignalHandlers(): void {
  process.on('SIGINT', handleInterrupt)
  process.on('SIGTERM', handleInterrupt)
  process.on('SIGPIPE', () => {
    // Ignore broken pipe errors
  })
}

function handleInterrupt(): void {
  interrupted = true
  process.exit(130)
}

// Check if interrupted and throw if so
export function checkInterrupt(): void {
  if (interrupted) {
    throw new InterruptError('Interrupted')
  }
}

// TTY detection utilities
export function isTTY(stream: 'stdin' | 'stdout' | 'stderr'): boolean {
  switch (stream) {
    case 'stdin':
      return process.stdin.isTTY === true
    case 'stdout':
      return process.stdout.isTTY === true
    case 'stderr':
      return process.stderr.isTTY === true
  }
}

export function isInteractive(): boolean {
  return process.stdin.isTTY === true && process.stdout.isTTY === true
}
