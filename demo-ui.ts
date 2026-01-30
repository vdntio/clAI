#!/usr/bin/env bun
// Demo script to test the interactive UI

import { renderUI, withSpinner, printCommand, printWarning } from './src/ui/index.js'
import type { Config } from './src/config/types.js'

const config: Config = {
  provider: { default: 'openrouter', fallback: [] },
  context: {
    maxFiles: 10,
    maxHistory: 3,
    redactPaths: false,
    redactUsername: false,
  },
  safety: {
    confirmDangerous: true,
    dangerousPatterns: [],
  },
  ui: {
    color: 'auto',
    interactive: true,
    numOptions: 3,
    promptTimeout: 0,
  },
  providers: {},
  quiet: false,
  verbose: 0,
  force: false,
  dryRun: false,
  offline: false,
  debug: false,
  instruction: 'demo',
}

const commands = [
  'find . -name "*.ts" -type f | wc -l',
  'git status --short',
  'ls -la src/',
]

const isDangerous = process.argv.includes('--dangerous')
const spinnerOnly = process.argv.includes('--spinner')
const outputOnly = process.argv.includes('--output')

async function main() {
  // Demo 1: Spinner
  if (spinnerOnly) {
    console.error('\n🔄 Spinner Demo\n')
    await withSpinner('Thinking...', async () => {
      await new Promise(r => setTimeout(r, 2000))
    })
    console.error('Done!\n')
    return
  }

  // Demo 2: Pretty output (non-interactive)
  if (outputOnly) {
    console.error('\n📤 Output Demo\n')
    printCommand('find . -name "*.ts" | wc -l')
    console.error('')
    printWarning('This command may be dangerous')
    printCommand('rm -rf /tmp/test', true)
    console.error('')
    return
  }

  // Demo 3: Interactive UI
  console.error('\n🎨 Interactive UI Demo\n')
  if (isDangerous) {
    console.error('Mode: Dangerous command\n')
  }

  const result = await renderUI({
    commands,
    config,
    isDangerous,
  })

  console.error(`\nResult: ${result.action} → ${result.command}\n`)
}

main().catch(console.error)
