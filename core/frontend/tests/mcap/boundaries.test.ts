import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'

import { describe, expect, it } from 'vitest'

const LIBRARY = path.resolve(__dirname, '../../src/libs/mcap')

describe('mcap boundaries', () => {
  it('imports no application module, so the recorder client passes in what it reads', () => {
    const files = readdirSync(LIBRARY, { recursive: true, encoding: 'utf8' }).filter((file) => file.endsWith('.ts'))
    const offenders = files.filter((file) => /['"]@\//.test(readFileSync(path.join(LIBRARY, file), 'utf8')))

    expect(files.length).toBeGreaterThan(0)
    expect(offenders).toEqual([])
  })
})
