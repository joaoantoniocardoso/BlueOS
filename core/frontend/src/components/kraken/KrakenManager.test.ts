import { expect, mock, test } from 'bun:test'

const calls: { url: string }[] = []
const sources = [
  { identifier: 'a', name: 'a', url: 'https://a', enabled: true, priority: 0, factory: false },
  { identifier: 'b', name: 'b', url: 'https://b', enabled: true, priority: 1, factory: false, error: 'backend is offline' },
]
mock.module('@/utils/api', () => ({
  default: async (config: { url: string }) => {
    calls.push(config)
    return { data: sources }
  },
}))

const { fetchManifestSources } = await import('./KrakenManager')

test('the source listing carries the load error of a source and downloads no data', async () => {
  const listed = await fetchManifestSources(false)
  expect(listed.map((source) => source.error)).toEqual([undefined, 'backend is offline'])
  expect(calls.map((call) => call.url)).toEqual(['/kraken/v2.0/manifest/?data=false&enabled=false'])
})
