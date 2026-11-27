import { beforeEach, expect, mock, test } from 'bun:test'

const calls: { method?: string, url: string, data?: any }[] = []
mock.module('@/utils/api', () => ({
  default: async (config: { method?: string, url: string, data?: any }) => {
    calls.push(config)
    return { data: [] }
  },
}))

const manager = await import('./KrakenManager')

beforeEach(() => {
  calls.length = 0
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
