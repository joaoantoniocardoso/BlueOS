export const DEFAULT_REFERENCE_DUT_TAGS = ['master', '1.4-dev'] as const

export interface FeatureAvailability {
  intro_commit: string,
  present_in_tags: string[],
  present_on_master: boolean,
  present_on_1_4_dev: boolean,
}

function normalizeDutTag(dutTag: string): string {
  return dutTag.startsWith('v') ? dutTag.slice(1) : dutTag
}

export function featurePresentOn(dutTag: string, availability: FeatureAvailability): boolean {
  const tag = normalizeDutTag(dutTag)
  if (tag === 'master') {
    return availability.present_on_master
  }
  if (tag === '1.4-dev') {
    return availability.present_on_1_4_dev
  }
  return availability.present_in_tags.includes(tag)
}

export function formatAvailabilitySkipReason(
  introCommit: string,
  firstTag: string | undefined,
  dutTag: string,
): string {
  const first = firstTag ?? '(none)'
  const short = introCommit.slice(0, 12)
  return `not present on ${dutTag} (intro ${short}; first tag ${first})`
}

export function availabilitySkip(
  dutTag: string,
  availability: FeatureAvailability,
): { introCommit: string, firstTag: string | undefined } | null {
  if (featurePresentOn(dutTag, availability)) {
    return null
  }
  return {
    introCommit: availability.intro_commit,
    firstTag: availability.present_in_tags[0],
  }
}

export function journeyAvailabilitySkipReason(
  dutTag: string,
  availability: FeatureAvailability,
): string | null {
  const skip = availabilitySkip(dutTag, availability)
  if (!skip) {
    return null
  }
  return formatAvailabilitySkipReason(skip.introCommit, skip.firstTag, dutTag)
}

export function isReferenceDutTag(dutTag: string, referenceDutTags: string[]): boolean {
  return referenceDutTags.includes(normalizeDutTag(dutTag))
}

export function precomputedTagsOnlyMessage(referenceDutTags: string[]): string {
  return `skip reasons are precomputed only for ${referenceDutTags.join(' and ')} in this snapshot.`
}

export function lookupPrecomputedSkipReason(
  skipReasons: Record<string, string | null>,
  dutTag: string,
  referenceDutTags: string[] = [...DEFAULT_REFERENCE_DUT_TAGS],
): string | null {
  const tag = normalizeDutTag(dutTag)
  if (!referenceDutTags.includes(tag)) {
    return precomputedTagsOnlyMessage(referenceDutTags)
  }
  return skipReasons[tag] ?? null
}
