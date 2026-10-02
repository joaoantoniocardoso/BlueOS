import {
  decodeCdrWithSchema,
  defaultMessageForSchema,
  encodeCdrWithSchema,
} from '@/libs/blueos-api/cdr'

import type { CdrCodec } from '../logic/types'

/* eslint-disable import/prefer-default-export */
export const cdrCodec: CdrCodec = {
  decode(schemaName, schemaText, payload) {
    return decodeCdrWithSchema(schemaName, schemaText, payload)
  },
  encode(schemaName, schemaText, message) {
    return encodeCdrWithSchema(schemaName, schemaText, message)
  },
  defaults(schemaName, schemaText) {
    return defaultMessageForSchema(schemaName, schemaText)
  },
}
