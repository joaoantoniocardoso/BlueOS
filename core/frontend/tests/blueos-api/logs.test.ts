import { readFileSync } from 'node:fs'
import path from 'node:path'

import { describe, expect, it } from 'vitest'

import { decodeCdr } from '@/libs/blueos-api/cdr'
import { cdrEncoding } from '@/libs/blueos-api/keys'
import { watchLogs } from '@/libs/blueos-api/logs'
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
