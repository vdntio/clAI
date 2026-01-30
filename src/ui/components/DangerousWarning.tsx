// src/ui/components/DangerousWarning.tsx
// Simple one-line warning for dangerous commands

import React from 'react'
import { Box, Text } from 'ink'

export interface DangerousWarningProps {
  message?: string
}

export function DangerousWarning({
  message = 'This command may modify or delete files',
}: DangerousWarningProps): React.ReactElement {
  return (
    <Box marginTop={1}>
      <Text color="yellow">⚠️  {message}</Text>
    </Box>
  )
}

export default DangerousWarning
