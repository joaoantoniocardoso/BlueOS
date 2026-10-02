import { readFileSync } from 'node:fs'
import path from 'node:path'

import { describe, expect, it } from 'vitest'

import { decodeCdr, encodeCdr } from '@/libs/blueos-api/cdr'

interface CdrVector {
  schema_name: string
  hex: string
  decoded: Record<string, unknown>
  category: string
  skip_encode_round_trip?: boolean
}

interface CdrVectorsFile {
  vectors: CdrVector[]
}

function decodeHex(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2)
  for (let index = 0; index < hex.length; index += 2) {
    bytes[index / 2] = Number.parseInt(hex.slice(index, index + 2), 16)
  }
  return bytes
}

function vectorsFile(): CdrVectorsFile {
  const filePath = path.resolve(__dirname, '../../../libs/idl/tests/vectors/cdr.json')
  return JSON.parse(readFileSync(filePath, 'utf8')) as CdrVectorsFile
}

describe('blueos-api CDR shared vectors', () => {
  const vectors = vectorsFile().vectors.filter((vector) => vector.schema_name.startsWith('blueos_')
    || vector.schema_name.startsWith('builtin_interfaces/')
    || vector.schema_name.startsWith('foxglove_msgs/')
    || vector.schema_name.startsWith('std_msgs/'))

  for (const vector of vectors) {
    it(`decodes ${vector.category} vector for ${vector.schema_name}`, () => {
      const payload = decodeHex(vector.hex)
      expect(decodeCdr(vector.schema_name as never, payload)).toEqual(vector.decoded)
    })

    if (!vector.skip_encode_round_trip) {
      it(`encodes ${vector.category} vector for ${vector.schema_name}`, () => {
        const payload = decodeHex(vector.hex)
        expect(encodeCdr(vector.schema_name as never, vector.decoded as never)).toEqual(payload)
      })
    }
  }
})
