import {
  describe, expect, it, vi,
} from 'vitest'

import type { Reply, Transport } from '@/libs/blueos-api/transport'
import {
  fetchInitialStateSamples,
  INSPECTOR_STATE_QUERY_SELECTORS,
} from '@/libs/zenoh-inspector/adapters/initial-state-queries'

describe('zenoh-source initial state queries', () => {
  it('never gets wider than state, settings and jobs selectors', () => {
    expect(INSPECTOR_STATE_QUERY_SELECTORS).toEqual([
      'blueos/v1/*/state/*',
      'blueos/v1/*/settings',
      'blueos/v1/*/jobs',
    ])
  })

  it('fetchInitialStateSamples queries only those selectors', async () => {
    const queriedKeys: string[] = []
    const transport: Pick<Transport, 'get'> = {
      get: vi.fn(async (key: string): Promise<Reply[]> => {
        queriedKeys.push(key)
        return []
      }),
    }
    await fetchInitialStateSamples(transport, {
      onSample: () => undefined,
      onRos2Liveliness: () => undefined,
      onBlueosServiceLiveliness: () => undefined,
      onError: () => undefined,
    }, () => 0)
    expect(queriedKeys).toEqual([...INSPECTOR_STATE_QUERY_SELECTORS])
  })
})
