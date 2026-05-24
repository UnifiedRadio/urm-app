import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { UrcProfile, Channel } from '../types/urc'
import { validateProfile } from '../api/config'

export const useChannelStore = defineStore('channels', () => {
  const profile = ref<UrcProfile | null>(null)
  const isDirty = ref(false)
  const validationErrors = ref<string[]>([])

  const channelCount = computed(() => profile.value?.channels.length ?? 0)

  function setProfile(p: UrcProfile) {
    profile.value = p
    isDirty.value = false
    validationErrors.value = []
  }

  function updateChannel(updated: Channel) {
    if (!profile.value) return
    const idx = profile.value.channels.findIndex(c => c.id === updated.id)
    if (idx >= 0) {
      profile.value.channels[idx] = updated
      isDirty.value = true
    }
  }

  function addChannel(channel: Channel) {
    if (!profile.value) return
    profile.value.channels.push(channel)
    isDirty.value = true
  }

  function removeChannel(id: string) {
    if (!profile.value) return
    profile.value.channels = profile.value.channels.filter(c => c.id !== id)
    isDirty.value = true
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
