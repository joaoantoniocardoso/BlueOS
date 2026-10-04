export interface VariationSummary {
  latestSeconds: number
  averageSeconds: number
  /** How much the value changed since the previous sample. */
  changeSeconds: number
  averageChangeSeconds: number
}

export interface VariationMeter {
  add(nowSeconds: number, valueSeconds: number): VariationSummary
}

interface VariationSample {
  atSeconds: number
  valueSeconds: number
  /** Null for the first sample, which has nothing to compare with. */
  changeSeconds: number | null
}

function average(values: number[]): number {
  return values.length > 0 ? values.reduce((sum, value) => sum + value, 0) / values.length : 0
}

/**
 * Tracks a value and its change between samples, averaged over a time window because players report at
 * different rates (per frame, per `timeupdate`). Fed with frame transit times (arrival minus source time), the
 * change is the RFC 3550 interarrival jitter, which a constant offset between the two clocks does not affect.
 */
export function createVariationMeter(windowSeconds: number): VariationMeter {
  let samples: VariationSample[] = []
  let previousSeconds: number | null = null
  return {
    add(nowSeconds: number, valueSeconds: number): VariationSummary {
      const changeSeconds = previousSeconds === null ? null : Math.abs(valueSeconds - previousSeconds)
      previousSeconds = valueSeconds
      samples = samples.filter((sample) => sample.atSeconds > nowSeconds - windowSeconds)
      samples.push({ atSeconds: nowSeconds, valueSeconds, changeSeconds })
      return {
        latestSeconds: valueSeconds,
        averageSeconds: average(samples.map((sample) => sample.valueSeconds)),
        changeSeconds: changeSeconds ?? 0,
        averageChangeSeconds: average(samples.flatMap((sample) => sample.changeSeconds ?? [])),
      }
    },
  }
}
