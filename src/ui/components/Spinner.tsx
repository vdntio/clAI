// src/ui/components/Spinner.tsx
// Animated loading spinner with smooth braille animation

import React from 'react'
import { Box, Text } from 'ink'
import { useAnimation } from '../hooks/useAnimation.js'
import { useTerminalSize } from '../hooks/useTerminalSize.js'
import { SPINNER_FRAMES, SPINNER_DOTS, COLORS } from '../types.js'

export interface SpinnerProps {
  /** Message to display next to spinner */
  message?: string
  /** Color of the spinner */
  color?: string
  /** Animation speed in ms */
  speed?: number
}

/**
 * Animated spinner component for loading states
 * Uses braille characters for smooth animation
 */
export function Spinner({
  message = 'Processing...',
  color = COLORS.command,
  speed = 80,
}: SpinnerProps): React.ReactElement {
  const { isWide } = useTerminalSize()

  // Use fancier dots animation on wide terminals
  const frames = isWide ? SPINNER_DOTS : SPINNER_FRAMES
  const frame = useAnimation(frames, speed)

  return (
    <Box>
      <Text color={color} bold>
        {frame}
      </Text>
      <Text color={color}> {message}</Text>
    </Box>
  )
}

export default Spinner
