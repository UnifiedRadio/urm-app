<template>
  <div class="app-shell">
    <AppSidebar />
    <div class="main-content">
      <TopBar />
      <component :is="currentPageComponent" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useUiStore } from './store/ui'
import AppSidebar from './components/shared/AppSidebar.vue'
import TopBar from './components/shared/TopBar.vue'
import Dashboard from './pages/Dashboard.vue'
import ChannelEditorPage from './pages/ChannelEditor.vue'
import DeviceManagerPage from './pages/DeviceManager.vue'
import TemplatesPage from './pages/Templates.vue'
import SharePage from './pages/Share.vue'
import TeamActivityPage from './pages/TeamActivity.vue'
import SettingsPage from './pages/Settings.vue'

const ui = useUiStore()

const pageMap = {
  'dashboard': Dashboard,
  'channel-editor': ChannelEditorPage,
  'device-manager': DeviceManagerPage,
  'templates': TemplatesPage,
  'share': SharePage,
  'team': TeamActivityPage,
  'settings': SettingsPage,
}

const currentPageComponent = computed(() => pageMap[ui.currentPage])
</script>
