/* eslint-disable import/prefer-default-export */
import type { EndpointInfo } from '@blueos-idl/messages'

import type {
  CdrCodec, DecodedPayload, SchemaProvider,
} from './types'

const JOB_FEEDBACK_LIST_SCHEMA = 'blueos_msgs/msg/JobFeedbackList'
const JOB_RESULT_SCHEMA = 'blueos_msgs/msg/JobResult'

// The `info` reply lists the part after the wrapper's own schema, as one more `MSG:` section.
const PART_SECTION_START = `\n${'='.repeat(80)}\nMSG: `

interface CarriedPart {
  name: string
  text: string
}

function carriedPart(endpoint: EndpointInfo | undefined, provider: SchemaProvider): CarriedPart | undefined {
  const wrapperText = provider.schemaText(endpoint?.interface_type ?? '')
  if (endpoint === undefined || wrapperText === undefined
    || !endpoint.schema.startsWith(`${wrapperText}${PART_SECTION_START}`)) {
    return undefined
  }
  const section = endpoint.schema.slice(wrapperText.length + PART_SECTION_START.length)
  const nameEnd = section.indexOf('\n')
  return nameEnd < 0
    ? { name: section, text: '' }
    : { name: section.slice(0, nameEnd), text: section.slice(nameEnd + 1) }
}

function decodeBytes(bytes: unknown, part: CarriedPart, codec: CdrCodec): unknown {
  if (!Array.isArray(bytes) || bytes.length === 0) {
    return bytes
  }
  if (part.text.trim() === '') {
    return {}
  }
  try {
    return codec.decode(part.name, part.text, Uint8Array.from(bytes as number[]))
  } catch {
    // A part that does not decode stays as the bytes it came as.
    return bytes
  }
}

/**
 * Decodes the `uint8[]` a `JobFeedbackList` or `JobResult` carries (D-12, D-36) with the part schema its key's `info`
 * endpoint appends to the wrapper's, so the inspector shows the Feedback and Job result fields of the Job type. Any
 * other payload, or a key whose endpoint lists no part, comes back as it is.
 */
export function unwrapJobPart(
  decoded: DecodedPayload,
  key: string,
  endpoints: EndpointInfo[],
  provider: SchemaProvider,
  codec: CdrCodec,
): DecodedPayload {
  if (decoded.kind !== 'cdr'
    || decoded.schemaName !== JOB_FEEDBACK_LIST_SCHEMA && decoded.schemaName !== JOB_RESULT_SCHEMA) {
    return decoded
  }
  const part = carriedPart(endpoints.find((endpoint) => endpoint.key === key), provider)
  if (part === undefined) {
    return decoded
  }

  const wrapper = decoded.value as Record<string, unknown>
  if (decoded.schemaName === JOB_RESULT_SCHEMA) {
    return { ...decoded, value: { ...wrapper, result: decodeBytes(wrapper.result, part, codec) } }
  }
  const jobs = (wrapper.jobs as Array<Record<string, unknown>>)
    .map((job) => ({ ...job, feedback: decodeBytes(job.feedback, part, codec) }))
  return { ...decoded, value: { ...wrapper, jobs } }
}
