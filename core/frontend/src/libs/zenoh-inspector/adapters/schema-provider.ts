/* eslint-disable import/no-extraneous-dependencies */
import { SCHEMAS } from '@blueos-idl/schemas'

import type { SchemaProvider } from '../logic/types'

export interface LazySchemaProvider extends SchemaProvider {
  ready(): Promise<void>
}

export function createSchemaProvider(): LazySchemaProvider {
  let catalogSchemas: Record<string, string> | null = null
  let catalogPromise: Promise<void> | null = null

  function loadCatalog(): Promise<void> {
    if (catalogPromise !== null) {
      return catalogPromise
    }
    catalogPromise = import('@blueos-idl/catalog').then((module) => {
      catalogSchemas = module.CATALOG_SCHEMAS as Record<string, string>
    })
    return catalogPromise
  }

  return {
    schemaText(schemaName: string): string | undefined {
      const blueosText = SCHEMAS[schemaName as keyof typeof SCHEMAS]
      if (blueosText !== undefined) {
        return blueosText
      }
      return catalogSchemas?.[schemaName]
    },

    ready(): Promise<void> {
      return loadCatalog()
    },
  }
}
