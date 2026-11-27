<template>
  <v-dialog
    :value="value"
    max-width="640px"
    @input="$emit('input', $event)"
  >
    <v-card>
      <v-card-title>{{ t`Choose the source of ${name}` }}</v-card-title>
      <v-card-text>
        <div class="mb-2">
          {{ t`More than one catalog lists this extension with a different repository.` }}
        </div>
        <v-list class="source-choice-list">
          <v-list-item-group
            v-model="selected_index"
            mandatory
          >
            <v-list-item
              v-for="candidate in candidates"
              :key="`${candidate.manifest}-${candidate.docker}`"
              :disabled="browse"
              class="source-choice-row"
            >
              <v-list-item-content>
                <v-list-item-title class="d-flex align-center">
                  {{ format(candidate).repo }}
                  <v-chip
                    v-if="isInstalled(candidate)"
                    x-small
                    color="success"
                    class="ml-2"
                  >
                    {{ t`installed now` }}
                  </v-chip>
                </v-list-item-title>
                <v-list-item-subtitle>{{ format(candidate).source }}</v-list-item-subtitle>
                <v-list-item-subtitle>{{ t`Tags` }}: {{ format(candidate).tags }}</v-list-item-subtitle>
              </v-list-item-content>
            </v-list-item>
          </v-list-item-group>
        </v-list>
        <v-alert
          v-if="!browse && is_switch"
          type="warning"
          dense
          text
          class="mt-3 mb-0"
        >
          {{ t`Other tags of the installed repo will be removed.` }}
        </v-alert>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn
          text
          @click="$emit('input', false)"
        >
          {{ browse ? t`Close` : t`Cancel` }}
        </v-btn>
        <v-btn
          v-if="!browse"
          color="primary"
          @click="$emit('choose', selected_candidate)"
        >
          {{ t`Choose` }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts">
import { t } from 'ttag'
import Vue, { PropType } from 'vue'

import { SourceCandidate } from '@/types/kraken'
import { formatCandidate, isInstalledCandidate, isSourceSwitch } from '@/utils/extension_source'

export default Vue.extend({
  name: 'ExtensionSourceChoiceDialog',
  props: {
    value: {
      type: Boolean,
      required: true,
    },
    name: {
      type: String,
      default: '',
    },
    candidates: {
      type: Array as PropType<SourceCandidate[]>,
      required: true,
    },
    installedDocker: {
      type: String as PropType<string | undefined>,
      default: undefined,
    },
    browse: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      selected_index: 0,
    }
  },
  computed: {
    selected_candidate(): SourceCandidate | undefined {
      return this.candidates[this.selected_index]
    },
    is_switch(): boolean {
      return this.selected_candidate !== undefined
        && isSourceSwitch(this.selected_candidate, this.installedDocker)
    },
  },
  watch: {
    value(open: boolean) {
      if (open) {
        const installed = this.candidates.findIndex((candidate) => this.isInstalled(candidate))
        this.selected_index = Math.max(installed, 0)
      }
    },
  },
  methods: {
    t,
    format: formatCandidate,
    isInstalled(candidate: SourceCandidate): boolean {
      return isInstalledCandidate(candidate, this.installedDocker)
    },
  },
})
</script>
