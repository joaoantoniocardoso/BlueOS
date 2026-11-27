import { expect, test } from 'bun:test'

import PullTracker from './pull_tracker'

const SEPARATOR = '|\n\n|'

function fragment(index: number, status: number, payload: object | null, error?: string): string {
  return JSON.stringify({
    fragment: index,
    status,
    data: payload ? btoa(JSON.stringify(payload)) : null,
    ...(error ? { error } : {}),
  })
}

function event(response: string) {
  return { target: { response } } as any
}

test('a layer error followed by the final error fragment reports one error', () => {
  const errors: string[] = []
  const tracker = new PullTracker(() => {}, (error) => errors.push(error))
  const layerError = fragment(0, 200, { id: 'abc', errorDetail: { message: 'no such image' } })
  const finalError = fragment(1, 500, null, 'no such image')

  tracker.digestNewData(event(layerError))
  tracker.digestNewData(event(`${layerError}${SEPARATOR}${finalError}`))
  tracker.digestNewData(event(`${layerError}${SEPARATOR}${finalError}`))

  expect(errors).toHaveLength(1)
  expect(tracker.error).toBe(errors[0])
})
