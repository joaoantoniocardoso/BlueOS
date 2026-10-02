import { readFileSync } from 'node:fs'
import path from 'node:path'

import { describe, expect, it } from 'vitest'

import {
  API_VERSION,
  cdrEncoding,
  commandKey,
  ENCODING_APPLICATION_CDR,
  eventKey,
  extensionLogKey,
  httpGatewayPrefix,
  infoQueryKey,
  jobsKey,
  KEY_PREFIX,
  logKey,
  queryKey,
  serviceLivelinessKey,
  settingsKey,
  stateKey,
  statusStateKey,
  TYPE_HASH_ATTACHMENT_KEY,
} from '@/libs/blueos-api/keys'

interface KeysVectors {
  constants: {
    api_version: string
    key_prefix: string
    encoding_application_cdr: string
    type_hash_attachment_key: string
  }
  service_liveliness_key: Array<{ service: string, expected: string }>
  command_key: Array<{ service: string, name: string, expected: string }>
  state_key: Array<{ service: string, name: string, expected: string }>
  event_key: Array<{ service: string, name: string, expected: string }>
  query_key: Array<{ service: string, name: string, expected: string }>
  jobs_key: Array<{ service: string, expected: string }>
  settings_key: Array<{ service: string, expected: string }>
  log_key: Array<{ service: string, expected: string }>
  extension_log_key: Array<{ service: string, extension_identifier: string, expected: string }>
  http_gateway_prefix: Array<{ service: string, expected: string }>
  status_state_key: Array<{ service: string, expected: string }>
  info_query_key: Array<{ service: string, expected: string }>
  cdr_encoding: Array<{ schema_name: string, expected: string }>
}

function vectorsFile(): KeysVectors {
  const filePath = path.resolve(__dirname, '../../../libs/api/tests/vectors/keys.json')
  return JSON.parse(readFileSync(filePath, 'utf8')) as KeysVectors
}

describe('blueos-api key shared vectors', () => {
  const vectors = vectorsFile()

  it('matches constant exports', () => {
    expect(API_VERSION).toBe(vectors.constants.api_version)
    expect(KEY_PREFIX).toBe(vectors.constants.key_prefix)
    expect(ENCODING_APPLICATION_CDR).toBe(vectors.constants.encoding_application_cdr)
    expect(TYPE_HASH_ATTACHMENT_KEY).toBe(vectors.constants.type_hash_attachment_key)
  })

  it('matches service liveliness keys', () => {
    for (const caseEntry of vectors.service_liveliness_key) {
      expect(serviceLivelinessKey(caseEntry.service)).toBe(caseEntry.expected)
    }
  })

  it('matches command keys', () => {
    for (const caseEntry of vectors.command_key) {
      expect(commandKey(caseEntry.service, caseEntry.name)).toBe(caseEntry.expected)
    }
  })

  it('matches state keys', () => {
    for (const caseEntry of vectors.state_key) {
      expect(stateKey(caseEntry.service, caseEntry.name)).toBe(caseEntry.expected)
    }
  })

  it('matches event keys', () => {
    for (const caseEntry of vectors.event_key) {
      expect(eventKey(caseEntry.service, caseEntry.name)).toBe(caseEntry.expected)
    }
  })

  it('matches query keys', () => {
    for (const caseEntry of vectors.query_key) {
      expect(queryKey(caseEntry.service, caseEntry.name)).toBe(caseEntry.expected)
    }
  })

  it('matches jobs keys', () => {
    for (const caseEntry of vectors.jobs_key) {
      expect(jobsKey(caseEntry.service)).toBe(caseEntry.expected)
    }
  })

  it('matches settings keys', () => {
    for (const caseEntry of vectors.settings_key) {
      expect(settingsKey(caseEntry.service)).toBe(caseEntry.expected)
    }
  })

  it('matches log keys', () => {
    for (const caseEntry of vectors.log_key) {
      expect(logKey(caseEntry.service)).toBe(caseEntry.expected)
    }
  })

  it('matches extension log keys', () => {
    for (const caseEntry of vectors.extension_log_key) {
      expect(extensionLogKey(caseEntry.service, caseEntry.extension_identifier)).toBe(caseEntry.expected)
    }
  })

  it('matches http gateway prefixes', () => {
    for (const caseEntry of vectors.http_gateway_prefix) {
      expect(httpGatewayPrefix(caseEntry.service)).toBe(caseEntry.expected)
    }
  })

  it('matches status state keys', () => {
    for (const caseEntry of vectors.status_state_key) {
      expect(statusStateKey(caseEntry.service)).toBe(caseEntry.expected)
    }
  })

  it('matches info query keys', () => {
    for (const caseEntry of vectors.info_query_key) {
      expect(infoQueryKey(caseEntry.service)).toBe(caseEntry.expected)
    }
  })

  it('matches CDR encodings', () => {
    for (const caseEntry of vectors.cdr_encoding) {
      expect(cdrEncoding(caseEntry.schema_name)).toBe(caseEntry.expected)
    }
  })
})
