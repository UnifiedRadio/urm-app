<template>
  <aside class="app-sidebar">
    <nav>
      <button v-for="item in navItems" :key="item.page"
        :class="{ active: ui.currentPage === item.page }"
        @click="ui.navigate(item.page)">
        {{ item.label }}
      </button>
    </nav>
  </aside>
</template>
<script setup lang="ts">
import { useUiStore, type AppPage } from '../../store/ui'
const ui = useUiStore()
const navItems: Array<{ label: string; page: AppPage }> = [
  { label: 'Dashboard', page: 'dashboard' },
  { label: 'Channels', page: 'channel-editor' },
  { label: 'Devices', page: 'device-manager' },
  { label: 'Templates', page: 'templates' },
  { label: 'Share', page: 'share' },
  { label: 'Team', page: 'team' },
  { label: 'Settings', page: 'settings' },
]
</script>
