import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UrcProfile, Channel, ValidationItem } from '../types/urc'
import { validateProfile } from '../api/config'

const CACHE_KEY = 'urm_last_profile'

function loadCached(): UrcProfile | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY)
    return raw ? JSON.parse(raw) : null
  } catch { return null }
}

export const useChannelStore = defineStore('channels', () => {
  const profile = ref<UrcProfile | null>(loadCached())
  const isDirty = ref(false)
  const validationErrors = ref<ValidationItem[]>([])

  const channelCount = computed(() => profile.value?.channels.length ?? 0)

  function persistCache() {
    if (profile.value) {
      try { localStorage.setItem(CACHE_KEY, JSON.stringify(profile.value)) } catch { /* quota */ }
    }
  }

  function setProfile(p: UrcProfile) {
    profile.value = p
    isDirty.value = false
    validationErrors.value = []
    persistCache()
  }

  function updateChannel(updated: Channel) {
    if (!profile.value) return
    const idx = profile.value.channels.findIndex(c => c.id === updated.id)
    if (idx >= 0) {
      profile.value.channels[idx] = updated
      isDirty.value = true
      persistCache()
    }
  }

  function addChannel(channel: Channel) {
    if (!profile.value) return
    profile.value.channels.push(channel)
    isDirty.value = true
    persistCache()
  }

  function removeChannel(id: string) {
    if (!profile.value) return
    profile.value.channels = profile.value.channels.filter(c => c.id !== id)
    isDirty.value = true
    persistCache()
  }

  function markClean() {
    isDirty.value = false
  }

  async function validate() {
    if (!profile.value) return false
    const result = await validateProfile(profile.value)
    validationErrors.value = result.errors
    return result.valid
  }

  return { profile, isDirty, validationErrors, channelCount, setProfile, updateChannel, addChannel, removeChannel, markClean, validate }
})
