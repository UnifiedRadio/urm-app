<template>
  <header class="top-bar">
    <span class="profile-name">{{ channelStore.profile?.name ?? 'No profile' }}</span>
    <span v-if="channelStore.isDirty" class="dirty-indicator">● unsaved</span>
    <span class="cloud-state">{{ auth.user?.email ?? (auth.isCloudConfigured ? 'signed out' : 'cloud not configured') }}</span>
  </header>
</template>
<script setup lang="ts">
import { useChannelStore } from '../../store/channels'
import { useAuthStore } from '../../store/auth'
const channelStore = useChannelStore()
const auth = useAuthStore()
</script>
