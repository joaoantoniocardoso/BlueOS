import {
  Encoding,
  ReplyError,
  type Sample as ZenohSample,
  SampleKind,
  type Session,
  ZBytes,
} from '@eclipse-zenoh/zenoh-ts'

import type { Reply, Sample, Transport } from './transport'

function fromZenoh(sample: ZenohSample): Sample {
  return {
    key: sample.keyexpr().toString(),
    payload: sample.payload().toBytes(),
    encoding: sample.encoding().toString(),
  }
}

/** The backbone through a zenoh-ts Session, such as the websocket Session of `@/libs/zenoh`. */
export default function zenohTransport(session: Session): Transport {
  return {
    async subscribe(key, onSample) {
      const subscriber = await session.declareSubscriber(key, {
        handler: (sample: ZenohSample) => {
          if (sample.kind() === SampleKind.PUT) {
            onSample(fromZenoh(sample))
          }
        },
      })
      return { close: () => subscriber.undeclare() }
    },

    async get(key, body, limit) {
      const receiver = await session.get(key, body && {
        payload: new ZBytes(body.payload),
        encoding: Encoding.fromString(body.encoding),
        attachment: body.attachment && new ZBytes(body.attachment),
      })
      const replies: Reply[] = []
      for await (const reply of receiver ?? []) {
        const result = reply.result()
        replies.push(result instanceof ReplyError
          ? { kind: 'error', payload: result.payload().toBytes(), encoding: result.encoding().toString() }
          : { kind: 'sample', sample: fromZenoh(result) })
        if (replies.length === limit) {
          break
        }
      }
      return replies
    },

    async subscribeLiveliness(key, onAlive) {
      const subscriber = await session.liveliness().declareSubscriber(key, {
        history: true,
        handler: (sample: ZenohSample) => {
          onAlive(sample.kind() === SampleKind.PUT)
        },
      })
      return { close: () => subscriber.undeclare() }
    },
  }
}
