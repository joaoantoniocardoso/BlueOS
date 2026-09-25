import { describe, expect, it } from 'vitest'

import { recordingUrl } from '@/libs/recorder'

describe('recordingUrl', () => {
  it('encodes each path segment', () => {
    expect(recordingUrl('folder/file name.mcap')).toBe('/userdata/recorder/folder/file%20name.mcap')
  })

  it('drops empty segments', () => {
    expect(recordingUrl('/folder//file.mcap')).toBe('/userdata/recorder/folder/file.mcap')
  })

  it('encodes hash and percent in segments', () => {
    expect(recordingUrl('folder/file#1.mcap')).toBe('/userdata/recorder/folder/file%231.mcap')
    expect(recordingUrl('folder/100%25.mcap')).toBe('/userdata/recorder/folder/100%2525.mcap')
  })
})
