<template>
  <v-card outlined class="mb-4">
    <v-card-title class="text-subtitle-1">
      Recorded files
    </v-card-title>
    <v-card-text>
      <p class="text-body-2 mb-3">
        Finished MCAP files and extracted video previews are managed by the existing Records tool
        (recorder_extractor). Use it to browse, download, and delete recordings on disk.
      </p>
      <v-btn color="primary" :to="{ path: '/tools/records' }">
        <v-icon left>
          mdi-folder-play
        </v-icon>
        Open Records
      </v-btn>
    </v-card-text>
  </v-card>
</template>

<script lang="ts">
export default {
  name: 'RecorderRecordingsLink',
}
</script>
