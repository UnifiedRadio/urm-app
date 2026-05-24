<template>
  <div class="backup-list">
    <div class="backup-list-header">
      <h3>Version History</h3>
      <button class="btn-sm" @click="emit('refresh')">Refresh</button>
    </div>
    <ul v-if="backups.length">
      <li v-for="b in backups" :key="b.id" class="backup-item">
        <div class="backup-info">
          <span class="backup-kind">{{ b.kind === 'profile_snapshot' ? 'Snapshot' : 'Device Image' }}</span>
          <span class="backup-time">{{ formatDate(b.created_at) }}</span>
          <span v-if="b.device_model" class="backup-model">{{ b.device_model }}</span>
          <span class="backup-size">{{ formatSize(b.size_bytes) }}</span>
        </div>
        <button
          v-if="b.kind === 'profile_snapshot'"
          class="btn-sm"
          @click="emit('restore', b.id)"
        >
          Restore
        </button>
      </li>
    </ul>
    <div v-else class="empty-backups">No snapshots yet. Click "Save Snapshot" after editing.</div>
  </div>
</template>

<script setup lang="ts">
import type { BackupMeta } from '../../types/urc'

defineProps<{ backups: BackupMeta[] }>()
const emit = defineEmits<{
  restore: [id: string]
  refresh: []
}>()

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString()
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}
</script>
