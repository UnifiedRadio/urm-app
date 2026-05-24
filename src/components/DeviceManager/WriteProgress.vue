<template>
  <div class="write-progress">
    <div class="step" v-for="step in steps" :key="step.label" :class="step.status">
      <span class="label">{{ step.label }}</span>
      <span class="status">{{ step.status }}</span>
    </div>
  </div>
</template>
<script setup lang="ts">
defineProps<{
  steps: Array<{ label: string; status: 'pending' | 'running' | 'ok' | 'error' }>
}>()
</script>
