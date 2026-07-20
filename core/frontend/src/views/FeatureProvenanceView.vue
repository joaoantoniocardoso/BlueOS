<template>
  <v-container fluid>
    <v-row class="mb-2" align="center">
      <v-col cols="12" md="6">
        <h2 class="text-h5">
          Feature Provenance
        </h2>
        <div v-if="snapshot" class="caption text--secondary">
          Snapshot {{ snapshot.generated_at }}
        </div>
      </v-col>
      <v-col cols="12" md="6">
        <v-select
          v-model="selectedJourneyId"
          :items="journeyItems"
          label="Journey"
          dense
          outlined
          hide-details
        />
      </v-col>
    </v-row>

    <v-alert
      v-if="loading"
      type="info"
      dense
      text
    >
      Loading provenance snapshot...
    </v-alert>
    <v-alert
      v-else-if="error"
      type="error"
      dense
    >
      {{ error }}
    </v-alert>

    <template v-else-if="selectedJourney">
      <v-alert
        v-if="dutTag"
        :type="dutAlertType"
        dense
        class="mb-4"
      >
        <span v-if="referenceDutTag">
          DUT <strong>{{ dutTag }}</strong>
          <span v-if="dutSource === 'live'"> (current version)</span>
          <span v-else> (from query)</span>
          <span v-if="activeSkipReason"> — {{ activeSkipReason }}</span>
          <span v-else-if="activeSkipReason === null"> — available on this tag</span>
        </span>
        <span v-else>
          DUT <strong>{{ dutTag }}</strong> — {{ unsupportedDutTagMessage }}
        </span>
      </v-alert>

      <v-row>
        <v-col cols="12" md="4">
          <v-card outlined>
            <v-card-title class="subtitle-1">
              Presence
            </v-card-title>
            <v-card-text>
              <v-chip
                class="ma-1"
                small
                :color="selectedJourney.presence.present_on_master ? 'success' : 'error'"
                text-color="white"
              >
                master
              </v-chip>
              <v-chip
                class="ma-1"
                small
                :color="selectedJourney.presence.present_on_1_4_dev ? 'success' : 'error'"
                text-color="white"
              >
                1.4-dev
              </v-chip>
              <v-chip class="ma-1" small>
                {{ selectedJourney.presence.present_in_tags.length }} release tags
              </v-chip>
              <div class="caption text--secondary mt-2">
                Intro {{ selectedJourney.intro_commit.slice(0, 12) }}
              </div>
            </v-card-text>
          </v-card>

          <v-card outlined class="mt-4">
            <v-card-title class="subtitle-1">
              Skip reasons
            </v-card-title>
            <v-card-text>
              <div
                v-for="tag in referenceDutTags"
                :key="tag"
                class="mb-2"
                :class="{ 'font-weight-bold': tag === referenceDutTag }"
              >
                <v-chip
                  x-small
                  class="mr-2"
                  :color="tag === referenceDutTag ? 'primary' : undefined"
                >
                  {{ tag }}
                </v-chip>
                <span v-if="selectedJourney.skip_reasons[tag]">
                  {{ selectedJourney.skip_reasons[tag] }}
                </span>
                <span v-else class="success--text">
                  available
                </span>
              </div>
            </v-card-text>
          </v-card>

          <v-card outlined class="mt-4">
            <v-card-title class="subtitle-1">
              Discovery paths
            </v-card-title>
            <v-card-text>
              <div
                v-for="path in selectedJourney.discovery_paths"
                :key="path"
                class="caption monospace"
              >
                {{ path }}
              </div>
            </v-card-text>
          </v-card>
        </v-col>

        <v-col cols="12" md="8">
          <v-card outlined>
            <v-card-title class="subtitle-1">
              Timeline
            </v-card-title>
            <v-card-text>
              <div
                v-for="section in timelineSections"
                :key="section.title"
              >
                <div v-if="section.items.length" class="mb-4">
                  <div class="subtitle-2 mb-1">
                    {{ section.title }}
                  </div>
                  <v-list dense class="py-0">
                    <v-list-item
                      v-for="item in section.items"
                      :key="`${section.title}-${item.number}`"
                      :href="item.url"
                      target="_blank"
                      rel="noopener"
                    >
                      <v-list-item-content>
                        <v-list-item-title>
                          {{ section.prefix }}{{ item.number }}: {{ item.title }}
                        </v-list-item-title>
                      </v-list-item-content>
                      <v-list-item-action>
                        <v-icon small>
                          mdi-open-in-new
                        </v-icon>
                      </v-list-item-action>
                    </v-list-item>
                  </v-list>
                </div>
              </div>
            </v-card-text>
          </v-card>
        </v-col>
      </v-row>
    </template>
  </v-container>
</template>

<script lang="ts">
import Vue from 'vue'

import {
  DEFAULT_REFERENCE_DUT_TAGS,
  isReferenceDutTag,
  lookupPrecomputedSkipReason,
  precomputedTagsOnlyMessage,
} from '@/utils/feature_provenance'
import { loadCurrentVersion } from '@/utils/version_chooser'

interface ProvenanceLink {
  number: number,
  title: string,
  url: string,
}

interface JourneyPresence {
  present_in_tags: string[],
  present_on_master: boolean,
  present_on_1_4_dev: boolean,
}

interface Journey {
  id: string,
  intro_commit: string,
  discovery_paths: string[],
  landing_prs: ProvenanceLink[],
  follow_up_prs: ProvenanceLink[],
  backport_prs: ProvenanceLink[],
  issues: ProvenanceLink[],
  presence: JourneyPresence,
  skip_reasons: Record<string, string | null>,
}

interface ProvenanceSnapshot {
  schema_version: number,
  generated_at: string,
  reference_dut_tags: string[],
  journeys: Journey[],
}

interface TimelineSection {
  title: string,
  prefix: string,
  items: ProvenanceLink[],
}

export default Vue.extend({
  name: 'FeatureProvenanceView',
  data() {
    return {
      loading: true,
      error: null as string | null,
      snapshot: null as ProvenanceSnapshot | null,
      selectedJourneyId: null as string | null,
      dutTag: null as string | null,
      dutSource: null as 'query' | 'live' | null,
    }
  },
  computed: {
    journeys(): Journey[] {
      return this.snapshot?.journeys ?? []
    },
    journeyItems(): { text: string, value: string }[] {
      return this.journeys.map((journey) => ({ text: journey.id, value: journey.id }))
    },
    selectedJourney(): Journey | null {
      return this.journeys.find((journey) => journey.id === this.selectedJourneyId) ?? null
    },
    referenceDutTags(): string[] {
      return this.snapshot?.reference_dut_tags ?? [...DEFAULT_REFERENCE_DUT_TAGS]
    },
    referenceDutTag(): string | null {
      if (!this.dutTag || !isReferenceDutTag(this.dutTag, this.referenceDutTags)) {
        return null
      }
      return this.dutTag
    },
    activeSkipReason(): string | null | undefined {
      if (!this.dutTag || !this.selectedJourney || !this.referenceDutTag) {
        return undefined
      }
      return lookupPrecomputedSkipReason(
        this.selectedJourney.skip_reasons,
        this.dutTag,
        this.referenceDutTags,
      )
    },
    unsupportedDutTagMessage(): string {
      return precomputedTagsOnlyMessage(this.referenceDutTags)
    },
    dutAlertType(): string {
      if (!this.referenceDutTag) {
        return 'info'
      }
      return this.activeSkipReason ? 'warning' : 'success'
    },
    timelineSections(): TimelineSection[] {
      if (!this.selectedJourney) {
        return []
      }
      return [
        { title: 'Landing PRs', prefix: 'PR #', items: this.selectedJourney.landing_prs },
        { title: 'Follow-up PRs', prefix: 'PR #', items: this.selectedJourney.follow_up_prs },
        { title: 'Backport PRs', prefix: 'PR #', items: this.selectedJourney.backport_prs },
        { title: 'Issues', prefix: '#', items: this.selectedJourney.issues },
      ]
    },
  },
  async mounted() {
    await Promise.all([this.loadSnapshot(), this.resolveDutTag()])
  },
  methods: {
    async loadSnapshot(): Promise<void> {
      this.loading = true
      this.error = null
      try {
        const response = await fetch('/assets/feature-provenance.json')
        if (!response.ok) {
          throw new Error(`Failed to load snapshot (${response.status})`)
        }
        const data = await response.json() as ProvenanceSnapshot
        this.snapshot = data
        this.selectedJourneyId = data.journeys[0]?.id ?? null
      } catch (err) {
        this.error = err instanceof Error ? err.message : 'Failed to load provenance snapshot'
      } finally {
        this.loading = false
      }
    },
    async resolveDutTag(): Promise<void> {
      const queryDut = this.$route.query.dut
      if (typeof queryDut === 'string' && queryDut.length > 0) {
        this.dutTag = queryDut
        this.dutSource = 'query'
        return
      }
      try {
        const version = await loadCurrentVersion()
        this.dutTag = version.tag
        this.dutSource = 'live'
      } catch {
        this.dutTag = null
        this.dutSource = null
      }
    },
  },
})
</script>

<style scoped>
.monospace {
  font-family: monospace;
  word-break: break-all;
}
</style>
