import { readFileSync } from 'node:fs'
import path from 'node:path'

import { describe, expect, it } from 'vitest'

import { decodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding, extensionLogKey } from '@/libs/blueos-api/keys'
import {
  extensionLogsRequestKey,
  requestExtensionLogs,
  watchExtensionLogs,
  watchLogs,
} from '@/libs/blueos-api/logs'
import { LOG_SCHEMA } from '@/libs/blueos-api/types'

import FakeTransport from './fake-transport'

interface CdrVector {
  schema_name: string
  hex: string
  decoded: Record<string, unknown>
  category: string
}

function decodeHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2)
  for (let index = 0; index < hex.length; index += 2) {
    bytes[index / 2] = Number.parseInt(hex.slice(index, index + 2), 16)
  }
  return bytes
}

function pythonProducerLogVector(): CdrVector {
  const filePath = path.resolve(__dirname, '../../../libs/idl/tests/vectors/cdr.json')
  const vectors = JSON.parse(readFileSync(filePath, 'utf8')) as { vectors: CdrVector[] }
  const matching = vectors.vectors.filter(
    (vector) => vector.schema_name === LOG_SCHEMA && vector.category === 'python_producer',
  )
  if (matching.length !== 1) {
    throw new Error('expected one python_producer Log vector')
  }
  return matching[0]
}

describe('watchLogs', () => {
  it('decodes CDR Log samples from Python and Rust producers', async () => {
    const vector = pythonProducerLogVector()
    const transport = new FakeTransport()
    const received: string[] = []

    const subscription = await watchLogs(transport, 'wifi-manager', {
      onLog: (entry) => {
        received.push(entry.message)
      },
    })

    transport.publish({
      key: 'blueos/v1/wifi-manager/log',
      payload: decodeHex(vector.hex),
      encoding: cdrEncoding(LOG_SCHEMA),
    })

    expect(received).toEqual([vector.decoded.message])
    await subscription.close()
  })

  it('decodes round-tripped Log payloads', () => {
    const vector = pythonProducerLogVector()
    const payload = decodeHex(vector.hex)
    expect(decodeCdr(LOG_SCHEMA, payload)).toEqual(vector.decoded)
  })
})

describe('watchExtensionLogs', () => {
  it('decodes CDR Log samples on the extension log key', async () => {
    const vector = pythonProducerLogVector()
    const transport = new FakeTransport()
    const received: string[] = []

    const subscription = await watchExtensionLogs(transport, 'kraken', 'my_ext', {
      onLog: (entry) => {
        received.push(entry.message)
      },
    })

    transport.publish({
      key: extensionLogKey('kraken', 'my_ext'),
      payload: decodeHex(vector.hex),
      encoding: cdrEncoding(LOG_SCHEMA),
    })

    expect(received).toEqual([vector.decoded.message])
    await subscription.close()
  })
})

describe('requestExtensionLogs', () => {
  it('decodes the Kraken extension logs request JSON payload', async () => {
    const transport = new FakeTransport()
    const requestPromise = requestExtensionLogs(transport, 'kraken', 'demo-ext')
    const pending = await transport.nextQuery()
    expect(pending.key).toBe(extensionLogsRequestKey('kraken', 'demo-ext'))
    pending.reply({
      kind: 'sample',
      sample: {
        key: pending.key,
        payload: new TextEncoder().encode(JSON.stringify({
          status: 'success',
          messages: [{ level: 2, message: 'INFO: started' }],
          total_lines: 1,
          topic: extensionLogKey('kraken', 'demo-ext'),
        })),
        encoding: 'application/json',
      },
    })
    const response = await requestPromise
    expect(response?.messages).toEqual([{ level: 2, message: 'INFO: started' }])
    expect(response?.topic).toBe(extensionLogKey('kraken', 'demo-ext'))
  })
})
