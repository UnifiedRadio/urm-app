import { defineStore } from 'pinia'
import { ref } from 'vue'

export type AppPage = 'dashboard' | 'channel-editor' | 'device-manager' | 'templates' | 'share' | 'team' | 'settings'

export const useUiStore = defineStore('ui', () => {
  const currentPage = ref<AppPage>('dashboard')
  const sidebarOpen = ref(true)
  const notifications = ref<Notification[]>([])

  function navigate(page: AppPage) {
    currentPage.value = page
  }

  function notify(message: string, type: Notification['type'] = 'info') {
    const n: Notification = { id: Date.now().toString(), message, type }
    notifications.value.push(n)
    setTimeout(() => dismiss(n.id), 5000)
  }

  function dismiss(id: string) {
    notifications.value = notifications.value.filter(n => n.id !== id)
  }

  return { currentPage, sidebarOpen, notifications, navigate, notify, dismiss }
})

interface Notification {
  id: string
  message: string
  type: 'info' | 'success' | 'warning' | 'error'
}
