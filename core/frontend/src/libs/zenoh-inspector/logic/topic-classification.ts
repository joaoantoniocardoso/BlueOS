import { ENCODING_APPLICATION_CDR } from '@/libs/blueos-api/keys'

const ZENOH_BYTES = 'zenoh/bytes'

/** CDR encapsulation header (CDR or PL_CDR), per D-24. */
export function hasCdrEncapsulationHeader(payload: Uint8Array): boolean {
  if (payload.length < 4) {
    return false
  }
  if (payload[0] === 0x00 && payload[1] === 0x01 && payload[2] === 0x00 && payload[3] === 0x00) {
    return true
  }
  if (payload[0] === 0x00 && payload[1] === 0x00 && payload[2] === 0x00 && payload[3] === 0x00) {
    return true
  }
  return false
}

export function encodingBase(encoding: string): string {
  const semicolon = encoding.indexOf(';')
  return semicolon < 0 ? encoding : encoding.slice(0, semicolon)
}

/** Encodings that may carry ROS 2 CDR when the payload has an encapsulation header. */
export function isAmbiguousCdrEncoding(base: string): boolean {
  return base === ZENOH_BYTES || base === '' || base === ENCODING_APPLICATION_CDR
}

/** Whether a sample should be decoded as CDR before schema resolution (D-24). */
export function payloadIsCdrCandidate(base: string, payload: Uint8Array): boolean {
  if (base === ENCODING_APPLICATION_CDR) {
    return hasCdrEncapsulationHeader(payload)
  }
  if (base === ZENOH_BYTES || base === '') {
    return hasCdrEncapsulationHeader(payload)
  }
  return false
}
