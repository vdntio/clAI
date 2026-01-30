// src/ui/hooks/useAnimation.ts
// Hook for smooth frame-based animations

import { useState, useEffect, useRef } from 'react'

/**
 * Hook that cycles through animation frames at a specified interval
 *
 * @param frames - Array of frame strings to cycle through
 * @param interval - Time between frames in milliseconds
 * @param enabled - Whether animation is active (default true)
 * @returns Current frame string
 */
export function useAnimation(
  frames: readonly string[],
  interval: number = 80,
  enabled: boolean = true
): string {
  const [frameIndex, setFrameIndex] = useState(0)
  const frameRef = useRef(0)

  useEffect(() => {
    if (!enabled || frames.length === 0) {
      return
    }

    const timer = setInterval(() => {
      frameRef.current = (frameRef.current + 1) % frames.length
      setFrameIndex(frameRef.current)
    }, interval)

    return () => {
      clearInterval(timer)
    }
  }, [frames, interval, enabled])

  return frames[frameIndex] ?? frames[0] ?? ''
}

/**
 * Hook for a pulsing animation effect (alternates between two states)
 *
 * @param interval - Time between pulses in milliseconds
 * @param enabled - Whether pulsing is active
 * @returns Boolean indicating current pulse state
 */
export function usePulse(interval: number = 500, enabled: boolean = true): boolean {
  const [isPulsed, setIsPulsed] = useState(false)

  useEffect(() => {
    if (!enabled) {
      return
    }

    const timer = setInterval(() => {
      setIsPulsed((prev) => !prev)
    }, interval)

    return () => {
      clearInterval(timer)
    }
  }, [interval, enabled])

  return isPulsed
}

/**
 * Hook for a typewriter-style reveal animation
 *
 * @param text - Full text to reveal
 * @param speed - Characters per second
 * @param enabled - Whether animation is active
 * @returns Currently visible portion of text
 */
export function useTypewriter(
  text: string,
  speed: number = 30,
  enabled: boolean = true
): string {
  const [visibleLength, setVisibleLength] = useState(enabled ? 0 : text.length)

  useEffect(() => {
    if (!enabled) {
      setVisibleLength(text.length)
      return
    }

    if (visibleLength >= text.length) {
      return
    }

    const interval = 1000 / speed
    const timer = setTimeout(() => {
      setVisibleLength((prev) => Math.min(prev + 1, text.length))
    }, interval)

    return () => {
      clearTimeout(timer)
    }
  }, [text, speed, enabled, visibleLength])

  // Reset when text changes
  useEffect(() => {
    if (enabled) {
      setVisibleLength(0)
    }
  }, [text, enabled])

  return text.slice(0, visibleLength)
}

export default useAnimation
