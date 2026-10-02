import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'

import { describe, expect, it } from 'vitest'

const LIBRARY = path.resolve(__dirname, '../../src/libs/blueos-api')

function imports(source: string): string[] {
  return [...source.matchAll(/(?:\bfrom|\bimport|\bimport\(|\brequire\()\s*['"]([^'"]+)['"]/g)]
    .map((match) => match[1])
}

function libraryImports(): Map<string, string[]> {
  const files = readdirSync(LIBRARY, { recursive: true, encoding: 'utf8' }).filter((file) => file.endsWith('.ts'))
  return new Map(files.map((file) => [file, imports(readFileSync(path.join(LIBRARY, file), 'utf8'))]))
}

function reachesVue(specifier: string): boolean {
  return /^(vue|vuex|vuetify|vue-[^/]+|@vue\/[^/]+)(\/|$)/.test(specifier)
    || specifier.endsWith('.vue')
    || specifier.startsWith('@/')
}

describe('blueos-api boundaries', () => {
  it('finds static, side-effect, dynamic and require imports', () => {
    const source = "import Vue from 'vue'\nimport 'vuetify/lib'\nconst store = import('vuex')\nrequire(\"@/store\")\n"

    expect(imports(source)).toEqual(['vue', 'vuetify/lib', 'vuex', '@/store'])
    expect(imports(source).every(reachesVue)).toBe(true)
  })

  it('imports no Vue package and no application module, so the Vue 3 migration only replaces a wrapper', () => {
    const offenders = [...libraryImports()]
      .flatMap(([file, specifiers]) => specifiers.filter(reachesVue).map((specifier) => `${file}: ${specifier}`))

    expect(libraryImports().size).toBeGreaterThan(0)
    expect(offenders).toEqual([])
  })

  it('decodes with rosmsg2-serialization and imports no WASM module', () => {
    const specifiers = [...libraryImports().values()].flat()

    expect(libraryImports().get('cdr.ts')).toContain('@foxglove/rosmsg2-serialization')
    expect(specifiers.filter((specifier) => /wasm/i.test(specifier))).toEqual([])
  })
})
