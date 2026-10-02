/* eslint-disable import/no-extraneous-dependencies, simple-import-sort/imports */
import path from 'path'

import { defineConfig } from 'vitest/config'

const codecData = path.resolve(__dirname, 'node_modules/mediabunny/dist/modules/src/codec-data.js')

export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@blueos-idl': path.resolve(__dirname, '../libs/idl/typescript'),
      'mediabunny/codec-data': codecData,
    },
  },
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
  },
})
