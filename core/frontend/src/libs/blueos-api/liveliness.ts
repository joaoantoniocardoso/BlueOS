import { serviceLivelinessKey } from './keys'
import type { Subscription, Transport } from './transport'

export interface ServiceAliveObserver {
  onAlive: (alive: boolean) => void
}

/** Follows a Service liveliness token (D-12). */
export function watchServiceAlive(
  transport: Transport,
  service: string,
  observer: ServiceAliveObserver,
): Promise<Subscription> {
  const key = serviceLivelinessKey(service)
  return transport.subscribeLiveliness(key, (alive) => observer.onAlive(alive))
}
