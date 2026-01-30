// Stdin input gathering for piped content

const MAX_STDIN_BYTES = 10 * 1024 // 10 KB

/**
 * Read stdin if it's not a TTY (i.e., data is being piped)
 * Returns up to 10KB of content as UTF-8 string
 * Returns undefined if stdin is a TTY or empty
 *
 * UTF-8 invalid sequences are replaced with replacement character (lossy decode)
 */
export async function readStdin(): Promise<string | undefined> {
  // Check if stdin is a TTY
  if (process.stdin.isTTY) {
    return undefined
  }

  // Check if stdin is readable (has data or will have data)
  // In test environments, stdin might not be a TTY but also not have any data
  if (!process.stdin.readable) {
    return undefined
  }

  // Check if stdin is already at EOF
  if (process.stdin.readableEnded) {
    return undefined
  }

  const chunks: Buffer[] = []
  let totalBytes = 0
  let hasReceivedData = false
  let endReceived = false

  return new Promise((resolve) => {
    // Set a timeout to prevent hanging if no data comes
    const timeout = setTimeout(() => {
      if (!hasReceivedData) {
        // No data received within timeout, assume empty stdin
        cleanup()
        resolve(undefined)
      }
    }, 100)

    function cleanup() {
      clearTimeout(timeout)
      process.stdin.removeAllListeners('data')
      process.stdin.removeAllListeners('end')
      process.stdin.removeAllListeners('error')
    }

    process.stdin.on('data', (chunk: Buffer) => {
      hasReceivedData = true
      const remainingBytes = MAX_STDIN_BYTES - totalBytes

      if (remainingBytes <= 0) {
        // We've already reached the limit, ignore further data
        return
      }

      // Only take what we need up to the limit
      const bytesToTake = Math.min(chunk.length, remainingBytes)
      chunks.push(chunk.subarray(0, bytesToTake))
      totalBytes += bytesToTake
    })

    process.stdin.on('end', () => {
      if (endReceived) return
      endReceived = true
      cleanup()

      if (chunks.length === 0) {
        resolve(undefined)
        return
      }

      const buffer = Buffer.concat(chunks)
      // Convert to UTF-8 with lossy replacement for invalid sequences
      const content = buffer.toString('utf8')
      resolve(content.length > 0 ? content : undefined)
    })

    process.stdin.on('error', () => {
      cleanup()
      // Non-fatal: return undefined on error
      resolve(undefined)
    })

    // Resume stdin if paused
    if (process.stdin.isPaused()) {
      process.stdin.resume()
    }
  })
}

/**
 * Check if stdin has piped data available
 * This is synchronous - useful for checking without reading
 */
export function hasPipedStdin(): boolean {
  return !process.stdin.isTTY && process.stdin.readable
}
