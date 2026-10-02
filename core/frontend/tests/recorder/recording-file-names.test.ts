import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

import { createdUnixSecondsFromFilename } from '@/libs/recorder/recording-file-timestamp'

type VectorEntry = {
  file_name: string
  file_time_unix_seconds: number
  created_unix_seconds: number
}

const vectorPath = join(
  dirname(fileURLToPath(import.meta.url)),
  '../../../services/recorder/logic/library/tests/vectors/recording_file_names.json',
)

describe('recording file name timestamps', () => {
  it('matches the shared Rust vector', () => {
    const entries = JSON.parse(readFileSync(vectorPath, 'utf8')) as VectorEntry[]
    for (const entry of entries) {
      expect(
        createdUnixSecondsFromFilename(entry.file_name, entry.file_time_unix_seconds),
      ).toBe(entry.created_unix_seconds)
    }
  })
})
