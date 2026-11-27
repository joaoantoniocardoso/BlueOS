<template>
  <v-dialog
    v-if="extension"
    width="700"
    :value="Boolean(extension)"
    persistent
    @input="showDialog"
    @click:outside="closeDialog"
    @keydown.ctrl.enter.prevent="saveExtension"
  >
    <v-card>
      <v-card-title class="d-flex align-center justify-space-between">
        <span>{{ t`${is_editing ? t`Edit` : (is_from_upload ? t`Configure Uploaded` : t`Create`)} Extension` }}</span>
        <v-btn
          icon
          x-small
          class="ml-2"
          :color="copySuccess ? 'success' : undefined"
          :title="t`Copy configuration`"
          @click="copyConfig"
        >
          <v-icon small>
            {{ copySuccess ? 'mdi-check' : 'mdi-content-copy' }}
          </v-icon>
        </v-btn>
      </v-card-title>

      <v-card-text
        class="d-flex flex-column"
        @paste="handlePaste"
      >
        <v-form
          ref="form"
          lazy-validation
        >
          <v-text-field
            v-model="new_extension.identifier"
            :label="t`Extension Identifier`"
            :placeholder="t`yourorganization.yourextension`"
            :disabled="is_editing"
            :rules="[validate_identifier]"
          />

          <v-text-field
            v-model="new_extension.name"
            :label="t`Extension Name`"
            :rules="[validate_name]"
          />

          <v-text-field
            v-model="new_extension.docker"
            :label="t`Docker image`"
            :disabled="is_editing"
            :rules="[validate_imagename]"
          />

          <v-text-field
            v-model="new_extension.tag"
            :label="t`Docker tag`"
            :rules="[validate_tag]"
            @paste="handleTagPaste"
          />

          <json-editor
            v-model="new_permissions"
            style="width:100%; min-height:400px"
          >
            <template
              v-if="is_reset_editing_permissions_visible"
              #controls
            >
              <v-btn
                v-tooltip="t`Reset to default permissions`"
                class="editor-control"
                icon
                color="white"
                @click="resetToDefaultPermissions"
              >
                <v-icon>mdi-restore</v-icon>
              </v-btn>
            </template>
          </json-editor>
        </v-form>
      </v-card-text>
      <v-card-actions
        class="pt-1"
      >
        <v-spacer />
        <v-btn
          color="primary"
          class="mr-4"
          :disabled="!valid_permissions"
          @click="saveExtension"
        >
          {{ is_editing ? t`Save` : (is_from_upload ? t`Install` : t`Create`) }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script lang="ts">
import Vue, { PropType } from 'vue'

import JsonEditor from '@/components/common/JsonEditor.vue'
import { copyToClipboard } from '@/cosmos'
import { InstalledExtensionData } from '@/types/kraken'
import { VForm } from '@/types/vuetify'
import {
  applyPastedConfig, isPermissionsObject, parsePermissions, validateIdentifier, validateImageName,
} from '@/utils/kraken_extension'

export default Vue.extend({
  name: 'ExtensionCreationModal',
  components: {
    JsonEditor,
  },
  model: {
    prop: 'extension',
    event: 'change',
  },
  props: {
    extension: {
      type: Object as PropType<InstalledExtensionData & { editing: boolean } | null>,
      default: null,
    },
    tempTag: {
      type: String,
      default: null,
    },
  },

  data() {
    return {
      new_extension: this.extension ?? {
        identifier: '',
        name: '',
        docker: '',
        tag: '',
        permissions: '{}',
        editing: false,
      } as InstalledExtensionData & { editing: boolean },
      new_permissions: parsePermissions(this.extension?.permissions),
      copySuccess: false,
    }
  },
  computed: {
    form(): VForm {
      return this.$refs.form as VForm
    },
    is_editing() {
      return this.extension?.editing ?? false
    },
    is_from_upload() {
      return Boolean(this.tempTag)
    },
    is_reset_editing_permissions_visible() {
      return this.new_permissions !== this.extension?.permissions
    },
    valid_permissions() {
      return isPermissionsObject(this.new_permissions)
    },
  },
  watch: {
    new_extension() {
      this.$emit('input', this.new_extension)
    },
    new_permissions(new_permissions: string) {
      this.new_extension.user_permissions = JSON.stringify(new_permissions)
    },
    extension() {
      if (this.extension) {
        this.new_extension = { ...this.extension }
      } else {
        this.new_extension = {
          identifier: '',
          name: '',
          docker: '',
          tag: '',
          permissions: '{}',
          editing: false,
        } as InstalledExtensionData & { editing: boolean }
      }
      this.populatePermissions()
    },
  },
  mounted() {
    this.populatePermissions()
  },
  methods: {
    populatePermissions() {
      const user_permissions = parsePermissions(this.extension?.user_permissions)
      if (Object.keys(user_permissions).length > 0) {
        this.new_permissions = user_permissions
        return
      }
      this.new_permissions = parsePermissions(this.extension?.permissions)
    },
    closeDialog() {
      this.new_extension = {
        identifier: '',
        name: '',
        docker: '',
        tag: '',
        permissions: '{}',
        editing: false,
      } as InstalledExtensionData & { editing: boolean }
      this.$emit('closed')
    },
    resetToDefaultPermissions() {
      if (this.new_extension) {
        this.new_permissions = parsePermissions(this.extension?.permissions)
      }
    },
    validate_identifier: validateIdentifier,
    validate_imagename: validateImageName,
    validate_name(input: string): (true | string) {
      // Trim only for checks so a space while typing the next word is not an error.
      const name = input.trim()
      if (name.length === 0) {
        return 'This field must not be empty.'
      }
      if (name.length > 128) {
        return 'This entry must fit within 128 characters.'
      }
      return true
    },
    validate_tag(input: string) {
      if (!input) {
        return 'Tag name must not be empty.'
      }
      if (/\s/.test(input)) {
        return 'Tag name must not include spaces.'
      }
      if (input.startsWith('-') || input.startsWith('.')) {
        return 'Tag name must not start with a period or a dash.'
      }
      if (input.length > 127) {
        return 'Tag name must be shorter than 128 characters.'
      }
      const re = /[^A-Za-z0-9\-_.]/
      if (re.test(input)) {
        return 'Only letters, numbers, dashes, periods, and underscores are allowed.'
      }
      return true
    },
    async saveExtension(): Promise<void> {
      if (isPermissionsObject(this.new_permissions)) {
        this.new_extension.user_permissions = JSON.stringify(this.new_permissions)
      }
      this.new_extension.name = this.new_extension.name.trim()
      if (this.form.validate() === true) {
        this.$emit('extensionChange', this.new_extension)
      }
    },
    showDialog(state: boolean) {
      if (this.form.validate() === true) {
        this.$emit('change', state)
      }
    },
    async copyConfig() {
      if (!this.new_extension) {
        return
      }

      const config = {
        identifier: this.new_extension.identifier,
        name: this.new_extension.name,
        docker: this.new_extension.docker,
        tag: this.new_extension.tag,
        permissions: this.new_extension.permissions !== '{}'
          ? this.new_extension.permissions : this.new_extension.user_permissions,
      }

      const jsonString = JSON.stringify(config, null, 2)
      const success = await copyToClipboard(jsonString)

      if (success) {
        this.copySuccess = true
        setTimeout(() => {
          this.copySuccess = false
        }, 2000) // Reset after 2 seconds
      }
    },
    handleTagPaste(event: ClipboardEvent) {
      const pastedText = event.clipboardData?.getData('text')?.trim()
      if (!pastedText) return

      const colonIdx = pastedText.lastIndexOf(':')
      if (colonIdx >= 0 && colonIdx < pastedText.length - 1) {
        event.preventDefault()
        this.new_extension.tag = pastedText.slice(colonIdx + 1)
      }
    },
    handlePaste(event: ClipboardEvent) {
      const pastedText = event.clipboardData?.getData('text')
      if (!pastedText) return

      if (!this.new_extension) {
        this.new_extension = {} as InstalledExtensionData & { editing: boolean }
      }
      const pasted = applyPastedConfig(this.new_extension, pastedText)
      if (!pasted) return

      event.preventDefault()
      if (pasted.permissions !== undefined) {
        this.new_permissions = pasted.permissions
      }
    },
  },
})
</script>
