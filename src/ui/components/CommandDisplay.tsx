// src/ui/components/CommandDisplay.tsx
// Clean, minimal command display

import React from 'react'
import { Box, Text } from 'ink'
import { useTerminalSize } from '../hooks/useTerminalSize.js'
import { formatCommand, formatCounter } from '../utils/formatCommand.js'

export interface CommandDisplayProps {
  command: string
  currentIndex: number
  totalCommands: number
  isDangerous?: boolean
}

export function CommandDisplay({
  command,
  currentIndex,
  totalCommands,
  isDangerous = false,
}: CommandDisplayProps): React.ReactElement {
  const { width } = useTerminalSize()

  const counterWidth = totalCommands > 1 ? 8 : 0
  const availableWidth = width - counterWidth - 4
  const formattedCommand = formatCommand(command, availableWidth)
  const counter = formatCounter(currentIndex, totalCommands)

  return (
    <Box>
      <Text color={isDangerous ? 'red' : 'green'} bold>$</Text>
      <Text> </Text>
      <Text color={isDangerous ? 'red' : 'cyan'} bold>{formattedCommand}</Text>
      {counter && (
        <>
          <Text>  </Text>
          <Text dimColor>{counter}</Text>
        </>
      )}
    </Box>
  )
}

export default CommandDisplay
