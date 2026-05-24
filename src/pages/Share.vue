<template>
  <div class="page page-share">
    <h1>Share</h1>

    <section>
      <h2>Create Radio Pack Link</h2>
      <p v-if="!auth.user" class="banner banner-warn">Sign in before creating share links.</p>
      <p v-if="!channelStore.profile">Load or create a profile before sharing.</p>
      <div class="form-row">
        <label>Expires in days
          <input v-model.number="expiresInDays" type="number" min="1" max="365" />
        </label>
        <button class="btn-primary" :disabled="!canCreate || loading" @click="createLink">
          {{ loading ? 'Creating...' : 'Create link' }}
        </button>
      </div>
      <p v-if="shareUrl">
        Share URL:
        <input :value="shareUrl" readonly />
      </p>
    </section>

    <section>
      <h2>Preview Shared Pack</h2>
      <div class="form-row">
        <label>Short code
          <input v-model="shortCode" placeholder="abc1234" />
        </label>
        <button class="btn" :disabled="!shortCode || loading" @click="loadPreview">Preview</button>
      </div>
      <div v-if="preview">
        <h3>{{ preview.profile.name }}</h3>
        <p>{{ preview.profile.channels.length }} channels · expires {{ preview.expires_at ? new Date(preview.expires_at).toLocaleDateString() : 'never' }}</p>
        <ul>
          <li v-for="ch in preview.profile.channels" :key="ch.id">
            {{ ch.name }} — {{ ch.rx_freq_mhz }} MHz
          </li>
        </ul>
        <button class="btn-primary" :disabled="!auth.user || loading" @click="importPreview">
          Import to my profiles
        </button>
      </div>
    </section>

    <p v-if="error" class="banner banner-error">{{ error }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAuthStore } from '../store/auth'
import { useChannelStore } from '../store/channels'
import { useCloudProfileStore } from '../store/cloudProfiles'
import type { SharePreview } from '../types/cloud'

const auth = useAuthStore()
const channelStore = useChannelStore()
const cloud = useCloudProfileStore()

const expiresInDays = ref(30)
const loading = ref(false)
const error = ref('')
const shareUrl = ref('')
const shortCode = ref('')
const preview = ref<SharePreview | null>(null)

const canCreate = computed(() => Boolean(auth.user && channelStore.profile))

async function ensureCloudProfile() {
  if (!channelStore.profile) throw new Error('No active profile.')
  const row = await cloud.saveProfile(channelStore.profile)
  channelStore.markClean()
  return row.id
}

async function createLink() {
  loading.value = true
  error.value = ''
  try {
    const profileId = await ensureCloudProfile()
    const result = await cloud.createShareLink(profileId, expiresInDays.value)
    shareUrl.value = result.url
    shortCode.value = result.short_code
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

async function loadPreview() {
  loading.value = true
  error.value = ''
  try {
    preview.value = await cloud.fetchSharePreview(shortCode.value.trim())
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}

async function importPreview() {
  if (!preview.value) return
  loading.value = true
  error.value = ''
  try {
    const imported = await cloud.importSharedProfile(preview.value)
    channelStore.setProfile(imported)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    loading.value = false
  }
}
</script>
