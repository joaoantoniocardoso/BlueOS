<template>
  <v-card-title class="d-flex align-center flex-wrap">
    <span>{{ title }}</span>
    <v-spacer />
    <v-chip :color="serviceAlive ? 'success' : 'error'" small>
      {{ serviceAlive ? 'live' : 'offline' }}
    </v-chip>
    <v-chip
      v-if="serviceVersion"
      class="ml-2"
      small
      outlined
    >
      v{{ serviceVersion }}
    </v-chip>
  </v-card-title>
</template>

<script lang="ts">
export default {
  name: 'RecorderServiceHeader',
  props: {
    title: {
      type: String,
      required: true,
    },
    serviceAlive: {
      type: Boolean,
      required: true,
    },
    serviceVersion: {
      type: String,
      default: '',
    },
  },
}
</script>
