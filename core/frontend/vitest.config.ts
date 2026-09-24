/* eslint-disable import/no-extraneous-dependencies, simple-import-sort/imports */
import path from 'path'

import { defineConfig } from 'vitest/config'

export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@blueos-idl': path.resolve(__dirname, '../libs/idl/typescript'),
    },
  },
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
  },
})
