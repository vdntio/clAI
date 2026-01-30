// src/ui/App.tsx
// Clean, minimal interactive UI

import React, { useState, useCallback, useEffect } from 'react'
import { Box, useInput, useApp, Text } from 'ink'
import { UserAction, type AppProps } from './types.js'
import { useTimeout } from './hooks/useTimeout.js'
import { CommandDisplay } from './components/CommandDisplay.js'
import { DangerousWarning } from './components/DangerousWarning.js'
import { ActionPrompt } from './components/ActionPrompt.js'

export function App({
  commands,
  isDangerous,
  config,
  onComplete,
}: AppProps): React.ReactElement {
  const { exit } = useApp()

  const [selectedIndex, setSelectedIndex] = useState(0)
  const [selectedAction, setSelectedAction] = useState(UserAction.Execute)

  const currentCommand = commands[selectedIndex] ?? ''
  const hasMultiple = commands.length > 1

  // Debug logging
  useEffect(() => {
    if (config.debug) {
      console.error(`[UI] Commands: ${commands.length}, Dangerous: ${isDangerous}`)
    }
  }, [commands.length, isDangerous, config.debug])

  // Handle completion
  const handleComplete = useCallback(
    (action: UserAction) => {
      if (config.debug) {
        console.error(`[UI] ${action}: ${currentCommand}`)
      }
      onComplete(action, currentCommand)
      exit()
    },
    [currentCommand, onComplete, exit, config.debug]
  )

  // Auto-abort timeout
  useTimeout(
    () => handleComplete(UserAction.Abort),
    config.ui.promptTimeout > 0 ? config.ui.promptTimeout : null
  )

  // Keyboard handling
  useInput((input, key) => {
    // Escape or Ctrl+C: cancel
    if (key.escape || (key.ctrl && input === 'c')) {
      handleComplete(UserAction.Abort)
      return
    }

    // Tab or arrows: cycle commands
    if (hasMultiple && (key.tab || key.leftArrow || key.rightArrow)) {
      const dir = key.leftArrow ? -1 : 1
      setSelectedIndex((i) => (i + dir + commands.length) % commands.length)
      return
    }

    // Up/Down: toggle action
    if (key.upArrow || key.downArrow) {
      setSelectedAction((a) =>
        a === UserAction.Execute ? UserAction.Abort : UserAction.Execute
      )
      return
    }

    // Enter: confirm
    if (key.return) {
      handleComplete(selectedAction)
      return
    }

    // Number keys for direct selection
    const num = parseInt(input, 10)
    if (hasMultiple && num >= 1 && num <= commands.length) {
      setSelectedIndex(num - 1)
    }
  })

  return (
    <Box flexDirection="column" paddingY={1}>
      {/* Command */}
      <CommandDisplay
        command={currentCommand}
        currentIndex={selectedIndex}
        totalCommands={commands.length}
        isDangerous={isDangerous}
      />

      {/* Tab hint for multiple commands */}
      {hasMultiple && (
        <Box marginTop={1}>
          <Text dimColor>Tab: next command</Text>
        </Box>
      )}

      {/* Warning for dangerous commands */}
      {isDangerous && <DangerousWarning />}

      {/* Actions */}
      <ActionPrompt
        selectedAction={selectedAction}
        isDangerous={isDangerous}
      />
    </Box>
  )
}

export default App
