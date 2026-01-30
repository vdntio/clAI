// src/output/index.ts
// Re-export all output-related functionality

export {
  ExecutionError,
  Errors,
  type ExecutionResult,
  type ValidationResult,
} from './types.js'

export {
  isRecursiveCall,
  validateCommand,
} from './validate.js'

export {
  getShell,
  executeCommand,
  type ExecuteOptions,
} from './execute.js'

// Re-export print functions from ui/output for convenience
export {
  printCommand,
  printWarning,
  printError,
  printSuccess,
  printInfo,
} from '../ui/output.js'
