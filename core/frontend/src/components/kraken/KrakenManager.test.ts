import {
  afterEach, expect, jest, mock, test,
} from 'bun:test'

import type { InstalledExtensionData } from '@/types/kraken'

const noop = (): void => {}

interface Captured {
  signal: AbortSignal
  timeout?: number
  onDownloadProgress: (event: object) => void
  resolve: () => void
  reject: (error: Error) => void
}
const calls: { method?: string, url: string, data?: any }[] = []
let captured: Captured | undefined
let hold = false
mock.module('../../utils/api', () => ({
  default: (config: Captured & { method?: string, url: string, data?: any }) => {
    calls.push(config)
    if (!hold) {
      return Promise.resolve({ data: [] })
    }
    return new Promise((resolve, reject) => {
      captured = config
      config.signal.addEventListener('abort', () => reject(new Error('canceled')))
      captured.resolve = resolve
      captured.reject = reject
    })
  },
}))
const manager = await import('./KrakenManager')
const { finalizeExtension, updateExtensionToVersion } = manager

afterEach(() => {
  jest.useRealTimers()
  captured = undefined
  hold = false
  calls.length = 0
})

async function started(call: () => Promise<void>): Promise<{ result: Promise<unknown>, config: Captured }> {
  hold = true
  const result = call().then(() => 'ok', (error) => error)
  // the request is issued after a few promise ticks, so wait for it instead of counting them
  for (let tick = 0; tick < 100 && !captured; tick += 1) {
    await Promise.resolve()
  }
  if (!captured) {
    throw new Error('the request was never issued')
  }
  return { result, config: captured }
}

test('aborts only after 120 s without progress, with a readable message', async () => {
  jest.useFakeTimers()
  const { result, config } = await started(() => updateExtensionToVersion('a.b', '1', noop))
  for (let i = 0; i < 5; i += 1) {
    jest.advanceTimersByTime(100000)
    config.onDownloadProgress({})
  }
  expect(config.signal.aborted).toBe(false)
  jest.advanceTimersByTime(119999)
  expect(config.signal.aborted).toBe(false)
  jest.advanceTimersByTime(1)
  expect(config.signal.aborted).toBe(true)
  expect((await result as Error).message).toBe('No progress from Docker for 120 s, aborted')
})

test('clears the timer on success', async () => {
  jest.useFakeTimers()
  const { result, config } = await started(() => updateExtensionToVersion('a.b', '1', noop))
  config.resolve()
  expect(await result).toBe('ok')
  expect(jest.getTimerCount()).toBe(0)
})

test('finalizeExtension is covered by the watchdog and has no wall-clock timeout', async () => {
  jest.useFakeTimers()
  const extension = {
    identifier: 'a.b', name: 'n', docker: 'd', tag: 't',
  } as InstalledExtensionData
  const { result, config } = await started(() => finalizeExtension(extension, 'tmp', noop))
  expect(config.timeout).toBeUndefined()
  jest.advanceTimersByTime(120000)
  expect((await result as Error).message).toContain('No progress')
})

test('a real request error is not reported as an idle timeout', async () => {
  jest.useFakeTimers()
  const { result, config } = await started(() => updateExtensionToVersion('a.b', '1', noop))
  config.reject(new Error('Request failed with status code 500'))
  expect((await result as Error).message).toBe('Request failed with status code 500')
  expect(jest.getTimerCount()).toBe(0)
})

test('extension URLs encode the identifier and the tag', async () => {
  const id = 'a/b?c#d'
  const tag = '1.0/../x'
  await manager.enableExtension(id, tag)
  await manager.disableExtension(id)
  await manager.uninstallExtension(id)
  await manager.restartExtension(id)
  await manager.updateExtensionToVersion(id, tag, () => {})
  const enc = 'a%2Fb%3Fc%23d'
  expect(calls.map((call) => call.url)).toEqual([
    `/kraken/v2.0/extension/${enc}/1.0%2F..%2Fx/enable`,
    `/kraken/v2.0/extension/${enc}/disable`,
    `/kraken/v2.0/extension/${enc}`,
    `/kraken/v2.0/extension/${enc}/restart`,
    `/kraken/v2.0/extension/${enc}/1.0%2F..%2Fx`,
  ])
})

test('install and finalize send an object for null permissions', async () => {
  const extension = {
    identifier: 'ab.cd', name: 'n', docker: 'd', tag: 't', permissions: null, user_permissions: null,
  } as any
  await manager.installExtension(extension, () => {})
  await manager.finalizeExtension(extension, 'temp', () => {})
  for (const call of calls) {
    expect(call.data.permissions).toBe('{}')
    expect(call.data.user_permissions).toBe('')
  }
})
