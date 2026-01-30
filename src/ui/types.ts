// src/ui/types.ts
// Types and enums for the interactive UI

import type { Config } from '../config/types.js'

/**
 * User action choices for command handling
 */
export enum UserAction {
  Execute = 'execute',
  Abort = 'abort',
}

/**
 * UI display phases
 */
export enum UIPhase {
  Loading = 'loading',
  Select = 'select',
  Done = 'done',
}

/**
 * UI state for the interactive prompt
 */
export interface UIState {
  phase: UIPhase
  commands: string[]
  selectedIndex: number
  selectedAction: UserAction
  isDangerous: boolean
  error?: string
}

/**
 * Props for the main App component
 */
export interface AppProps {
  commands: string[]
  isDangerous: boolean
  config: Config
  onComplete: (action: UserAction, command: string) => void
}

/**
 * Options for renderUI function
 */
export interface RenderOptions {
  commands: string[]
  config: Config
  isDangerous: boolean
}

/**
 * Result from renderUI function
 */
export interface RenderResult {
  action: UserAction
  command: string
}

/**
 * Terminal size breakpoints
 */
export interface TerminalBreakpoints {
  isNarrow: boolean  // < 60 columns
  isMedium: boolean  // 60-100 columns
  isWide: boolean    // > 100 columns
}

/**
 * Terminal dimensions with breakpoints
 */
export interface TerminalSize extends TerminalBreakpoints {
  width: number
  height: number
}

/**
 * Animation frame configuration
 */
export interface AnimationConfig {
  frames: readonly string[]
  interval: number
}

/**
 * Color palette for the UI
 */
export const COLORS = {
  // Primary colors
  command: '#00d9ff',      // Bright cyan for commands
  commandDim: '#0099b3',   // Dimmed cyan
  prompt: '#00ff88',       // Green for $ prompt

  // Action colors
  execute: '#00ff88',      // Green for execute
  executeDanger: '#ff4444', // Red for dangerous execute
  abort: '#ffaa00',        // Orange/yellow for abort

  // Warning colors
  danger: '#ff4444',       // Red for danger
  dangerBg: '#330000',     // Dark red background
  warning: '#ffaa00',      // Yellow/orange warning

  // UI chrome
  border: '#444444',       // Subtle border
  borderFocus: '#666666',  // Focused border
  dim: '#666666',          // Dimmed text
  highlight: '#ffffff',    // Highlighted text

  // Selection
  selected: '#00ff88',     // Selected item
  selectedBg: '#003311',   // Selected background
} as const

/**
 * Spinner animation frames (smooth braille animation)
 */
export const SPINNER_FRAMES = [
  '⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'
] as const

/**
 * Alternative spinner for wider terminals (dots animation)
 */
export const SPINNER_DOTS = [
  '⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'
] as const

/**
 * Pulsing animation frames for attention
 */
export const PULSE_FRAMES = ['●', '○'] as const
