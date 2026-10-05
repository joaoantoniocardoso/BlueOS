import { describe, expect, it } from 'vitest'

import { createVariationMeter } from '@/libs/zenoh-inspector/logic/variation-meter'

describe('variation meter', () => {
  it('reports no change for the first sample', () => {
    const meter = createVariationMeter(2)

    expect(meter.add(0, 0.1)).toEqual({
      latestSeconds: 0.1, averageSeconds: 0.1, changeSeconds: 0, averageChangeSeconds: 0,
    })
  })

  it('takes the change from the previous sample and leaves the first one out of its average', () => {
    const meter = createVariationMeter(2)
    meter.add(0, 0.1)

    const summary = meter.add(0.1, 0.04)

    expect(summary.latestSeconds).toBe(0.04)
    expect(summary.averageSeconds).toBeCloseTo(0.07)
    expect(summary.changeSeconds).toBeCloseTo(0.06)
    expect(summary.averageChangeSeconds).toBeCloseTo(0.06)
  })

  it('averages only the samples inside the window', () => {
    const meter = createVariationMeter(2)
    meter.add(0, 1)
    meter.add(1, 0.1)
    meter.add(2, 0.1)

    const summary = meter.add(3, 0.1)

    expect(summary.averageSeconds).toBeCloseTo(0.1)
    expect(summary.averageChangeSeconds).toBeCloseTo(0)
  })

  it('gives transit jitter that ignores a constant clock offset', () => {
    const meter = createVariationMeter(2)
    const clockOffsetSeconds = 1000
    const frames = [[0, 0.05], [0.033, 0.083], [0.066, 0.136]]

    const summaries = frames.map(([sourceSeconds, arrivalSeconds]) => {
      const delaySeconds = arrivalSeconds - (sourceSeconds + clockOffsetSeconds)
      return meter.add(arrivalSeconds, delaySeconds)
    })

    expect(summaries[1].changeSeconds).toBeCloseTo(0)
    expect(summaries[2].changeSeconds).toBeCloseTo(0.02)
    expect(summaries[2].averageChangeSeconds).toBeCloseTo(0.01)
  })
})
