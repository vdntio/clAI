// src/ui/hooks/useTimeout.ts
// Custom hook for configurable timeout handling

import { useEffect, useRef } from 'react'

/**
 * Hook that executes a callback after a specified delay
 * Automatically clears on unmount or when delay changes
 *
 * @param callback - Function to execute when timer expires
 * @param delay - Delay in milliseconds, or null/0 to disable
 */
export function useTimeout(
  callback: () => void,
  delay: number | null | undefined
): void {
  const callbackRef = useRef(callback)

  // Update callback ref when callback changes (avoids stale closures)
  useEffect(() => {
    callbackRef.current = callback
  }, [callback])

  useEffect(() => {
    // Don't set timeout if delay is null, undefined, or <= 0
    if (delay === null || delay === undefined || delay <= 0) {
      return
    }

    const timer = setTimeout(() => {
      callbackRef.current()
    }, delay)

    // Cleanup on unmount or delay change
    return () => {
      clearTimeout(timer)
    }
  }, [delay])
}

export default useTimeout
