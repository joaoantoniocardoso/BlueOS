import type { FrameScheduler } from './types'

export interface Coalescer<T> {
  push(key: string, value: T): void
  flushNow(key: string, value: T): void
  setSelectedKey(key: string | null): void
  dispose(): void
}

export function createCoalescer<T>(
  scheduler: FrameScheduler,
  onFlush: (batch: Record<string, T>) => void,
): Coalescer<T> {
  let pending = Object.create(null) as Record<string, T>
  let frameHandle: unknown | null = null
  let selectedKey: string | null = null

  function flush(): void {
    frameHandle = null
    const batch = pending
    pending = Object.create(null) as Record<string, T>
    if (Object.keys(batch).length > 0) {
      onFlush(batch)
    }
  }

  function scheduleFlush(): void {
    if (frameHandle !== null) {
      return
    }
    frameHandle = scheduler.schedule(flush)
  }

  return {
    push(key: string, value: T): void {
      if (selectedKey !== null && key === selectedKey) {
        onFlush({ [key]: value })
        delete pending[key]
        return
      }
      pending[key] = value
      scheduleFlush()
    },

    flushNow(key: string, value: T): void {
      onFlush({ [key]: value })
      delete pending[key]
    },

    setSelectedKey(key: string | null): void {
      selectedKey = key
    },

    dispose(): void {
      if (frameHandle !== null) {
        scheduler.cancel(frameHandle)
        frameHandle = null
      }
      pending = Object.create(null) as Record<string, T>
      selectedKey = null
    },
  }
}
