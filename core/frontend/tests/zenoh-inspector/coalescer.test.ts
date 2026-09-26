import {
  describe, expect, it, vi,
} from 'vitest'

import { createCoalescer } from '@/libs/zenoh-inspector/logic/coalescer'
import type { FrameScheduler } from '@/libs/zenoh-inspector/logic/types'

describe('coalescer', () => {
  it('batches by key and flushes on schedule', () => {
    const callbacks: Array<() => void> = []
    const scheduler: FrameScheduler = {
      schedule(callback: () => void): number {
        callbacks.push(callback)
        return callbacks.length
      },
      cancel(): void {
        /* no-op */
      },
    }
    const onFlush = vi.fn()
    const coalescer = createCoalescer(scheduler, onFlush)

    coalescer.push('a', 1)
    coalescer.push('a', 2)
    coalescer.push('b', 3)
    expect(onFlush).not.toHaveBeenCalled()
    callbacks[0]()
    expect(onFlush).toHaveBeenCalledWith({ a: 2, b: 3 })

    coalescer.dispose()
  })

  it('delivers the selected key immediately', () => {
    const scheduler: FrameScheduler = {
      schedule(): number {
        return 1
      },
      cancel(): void {
        /* no-op */
      },
    }
    const onFlush = vi.fn()
    const coalescer = createCoalescer(scheduler, onFlush)
    coalescer.setSelectedKey('live')
    coalescer.push('live', 'frame')
    expect(onFlush).toHaveBeenCalledWith({ live: 'frame' })
    coalescer.dispose()
  })
})
