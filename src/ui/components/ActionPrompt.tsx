// src/ui/components/ActionPrompt.tsx
// Simple, clean action buttons

import React from 'react'
import { Box, Text } from 'ink'
import { UserAction } from '../types.js'

export interface ActionPromptProps {
  selectedAction: UserAction
  isDangerous: boolean
}

export function ActionPrompt({
  selectedAction,
  isDangerous,
}: ActionPromptProps): React.ReactElement {
  const isExecute = selectedAction === UserAction.Execute
  const executeColor = isDangerous ? 'red' : 'green'

  return (
    <Box flexDirection="column" marginTop={1}>
      {/* Action buttons */}
      <Box gap={2}>
        <Text
          color={isExecute ? 'black' : executeColor}
          backgroundColor={isExecute ? executeColor : undefined}
          bold={isExecute}
        >
          {isExecute ? ' ▶ Run ' : '   Run '}
        </Text>
        <Text
          color={!isExecute ? 'black' : 'yellow'}
          backgroundColor={!isExecute ? 'yellow' : undefined}
          bold={!isExecute}
        >
          {!isExecute ? ' ▶ Cancel ' : '   Cancel '}
        </Text>
      </Box>

      {/* Simple hints */}
      <Box marginTop={1}>
        <Text dimColor>
          ↑↓ select  Enter confirm  Esc cancel
        </Text>
      </Box>
    </Box>
  )
}

export default ActionPrompt
