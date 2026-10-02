import { KEY_PREFIX } from '@/libs/blueos-api/keys'
import type { Transport } from '@/libs/blueos-api/transport'

import type { InspectorSourceHandlers } from '../inspector-controller'
import type { SampleRecord } from '../logic/types'

// Initial query selectors only; widening to command keys would execute Commands (D-24).
export const INSPECTOR_STATE_QUERY_SELECTORS = [
  `${KEY_PREFIX}/*/state/*`,
  `${KEY_PREFIX}/*/settings`,
  `${KEY_PREFIX}/*/jobs`,
] as const

function transportSampleToRecord(
  sample: { key: string, payload: Uint8Array, encoding: string },
  clock: () => number,
): SampleRecord {
  return {
    key: sample.key,
    payload: sample.payload,
    encoding: sample.encoding,
    receivedAt: clock(),
    kind: 'put',
  }
}

export async function fetchInitialStateSamples(
  transport: Pick<Transport, 'get'>,
  handlers: InspectorSourceHandlers,
  clock: () => number,
): Promise<void> {
  for (const selector of INSPECTOR_STATE_QUERY_SELECTORS) {
    // eslint-disable-next-line no-await-in-loop
    const replies = await transport.get(selector)
    for (const reply of replies) {
      if (reply.kind === 'sample') {
        handlers.onSample(transportSampleToRecord(reply.sample, clock))
      }
    }
  }
}
