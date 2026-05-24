<template>
  <div class="dmr-form">
    <!-- DMR fields — Phase 6 only. Analog MVP does not render this form. -->
    <p class="phase-note">DMR channel configuration — available in Phase 6</p>
  </div>
</template>
<script setup lang="ts">
import type { DmrFields } from '../../types/urc'
defineProps<{ modelValue?: DmrFields }>()
</script>
