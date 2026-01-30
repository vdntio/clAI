// src/ui/hooks/useTerminalSize.ts
// Hook for responsive terminal dimensions with breakpoints

import { useState, useEffect } from 'react'
import type { TerminalSize } from '../types.js'

// Breakpoint thresholds
const NARROW_THRESHOLD = 60
const WIDE_THRESHOLD = 100

/**
 * Get current terminal dimensions
 * Falls back to 80x24 if not available (standard terminal size)
 */
function getTerminalSize(): { width: number; height: number } {
  return {
    width: process.stdout.columns || 80,
    height: process.stdout.rows || 24,
  }
}

/**
 * Calculate breakpoints from width
 */
function getBreakpoints(width: number) {
  return {
    isNarrow: width < NARROW_THRESHOLD,
    isMedium: width >= NARROW_THRESHOLD && width < WIDE_THRESHOLD,
    isWide: width >= WIDE_THRESHOLD,
  }
}

/**
 * Hook that returns terminal dimensions with responsive breakpoints
 * Automatically updates on terminal resize
 *
 * Breakpoints:
 * - isNarrow: < 60 columns (compact layout, aggressive truncation)
 * - isMedium: 60-100 columns (standard layout)
 * - isWide: >= 100 columns (full layout with borders and padding)
 */
export function useTerminalSize(): TerminalSize {
  const [size, setSize] = useState(getTerminalSize())

  useEffect(() => {
    const handleResize = () => {
      setSize(getTerminalSize())
    }

    // Listen for terminal resize events
    process.stdout.on('resize', handleResize)

    // Cleanup listener on unmount
    return () => {
      process.stdout.off('resize', handleResize)
    }
  }, [])

  const breakpoints = getBreakpoints(size.width)

  return {
    width: size.width,
    height: size.height,
    ...breakpoints,
  }
}

export default useTerminalSize
