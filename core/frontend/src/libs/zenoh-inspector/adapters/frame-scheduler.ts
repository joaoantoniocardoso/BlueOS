import type { FrameScheduler } from '../logic/types'

/* eslint-disable import/prefer-default-export */
export const browserFrameScheduler: FrameScheduler = {
  schedule(callback: () => void): number {
    return requestAnimationFrame(callback)
  },
  cancel(handle: unknown): void {
    cancelAnimationFrame(handle as number)
  },
}
