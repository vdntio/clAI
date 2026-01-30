import { Command, InvalidArgumentError } from 'commander'
import { UsageError } from '../error/index.js'

export type ColorMode = 'auto' | 'always' | 'never'

export interface Cli {
  instruction: string
  model?: string
  provider?: string
  quiet: boolean
  verbose: number
  noColor: boolean
  color: ColorMode
  interactive: boolean
  force: boolean
  dryRun: boolean
  context?: string
  offline: boolean
  numOptions: number
  debug: boolean
  debugFile?: string
}

function parseNumOptions(value: string): number {
  const num = parseInt(value, 10)
  if (isNaN(num)) {
    throw new InvalidArgumentError('Must be a number')
  }
  // Clamp to 1-10
  return Math.max(1, Math.min(10, num))
}

function parseColorMode(value: string): ColorMode {
  if (value === 'auto' || value === 'always' || value === 'never') {
    return value
  }
  throw new InvalidArgumentError('Must be auto, always, or never')
}

export function parseCli(argv: string[] = process.argv): Cli {
  const program = new Command()

  program
    .name('clai')
    .description(
      'AI-powered CLI that converts natural language into executable shell commands'
    )
    .version('0.1.0', '-V, --version', 'Output the version number')
    .argument('[instruction]', 'Natural language instruction')
    .option('-m, --model <model>', 'Override AI model')
    .option('-p, --provider <provider>', 'Override AI provider')
    .option('-q, --quiet', 'Minimal output', false)
    .option(
      '-v, --verbose',
      'Increase verbosity (can be repeated)',
      (_, prev) => prev + 1,
      0
    )
    .option('--no-color', 'Disable color output')
    .option(
      '--color <mode>',
      'Color mode: auto, always, never',
      parseColorMode,
      'auto'
    )
    .option(
      '-i, --interactive',
      'Interactive mode (prompt execute/copy/abort)',
      false
    )
    .option('-f, --force', 'Skip dangerous command confirmation', false)
    .option('-n, --dry-run', 'Only print command(s), no execute', false)
    .option('-c, --context <file>', 'Optional context file path')
    .option('--offline', 'Offline mode (not implemented)', false)
    .option(
      '-o, --options <count>',
      'Number of command options (1-10)',
      parseNumOptions,
      1
    )
    .option('-d, --debug', 'Print prompt/request to stderr', false)
    .option('--debug-file [path]', 'Enable file logging (optional path)')
    .configureOutput({
      writeOut: (str) => process.stdout.write(str),
      writeErr: (str) => process.stderr.write(str),
      outputError: (str, write) => write(`Error: ${str}`),
    })

  program.parse(argv)

  const opts = program.opts()
  const args = program.args

  // instruction is required unless help/version was shown
  const instruction = args[0]
  if (!instruction) {
    throw new UsageError('missing required argument: instruction')
  }

  // --no-color overrides --color
  const noColor = opts.color === false // commander sets this when --no-color is used
  const colorMode: ColorMode = noColor ? 'never' : (opts.color as ColorMode)

  return {
    instruction,
    model: opts.model,
    provider: opts.provider,
    quiet: opts.quiet,
    verbose: opts.verbose,
    noColor,
    color: colorMode,
    interactive: opts.interactive,
    force: opts.force,
    dryRun: opts.dryRun,
    context: opts.context,
    offline: opts.offline,
    numOptions: opts.options,
    debug: opts.debug,
    debugFile: opts.debugFile,
  }
}
