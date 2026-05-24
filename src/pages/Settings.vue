<template>
  <div class="page page-settings">
    <h1>Settings</h1>

    <section>
      <h2>Account</h2>
      <p v-if="!auth.isCloudConfigured" class="banner banner-warn">
        Supabase is not configured. Set VITE_SUPABASE_URL and VITE_SUPABASE_ANON_KEY.
      </p>
      <div v-if="auth.user">
        <p>Signed in as {{ auth.user.email }}</p>
        <button class="btn" @click="auth.signOut()">Sign out</button>
      </div>
      <form v-else @submit.prevent="signIn">
        <label>Email
          <input v-model="email" type="email" required autocomplete="email" />
        </label>
        <label>Password
          <input v-model="password" type="password" required autocomplete="current-password" />
        </label>
        <div class="form-row">
          <button class="btn-primary" type="submit" :disabled="auth.loading || !auth.isCloudConfigured">
            {{ auth.loading ? 'Signing in...' : 'Sign in' }}
          </button>
          <button class="btn" type="button" :disabled="auth.loading || !auth.isCloudConfigured" @click="signUp">
            Create account
          </button>
        </div>
      </form>
      <p v-if="auth.error" class="banner banner-error">{{ auth.error }}</p>
    </section>

    <section>
      <h2>Cloud Profiles</h2>
      <div class="form-row">
        <button class="btn" :disabled="!auth.user || cloud.loading" @click="cloud.loadProfiles()">
          Refresh profiles
        </button>
        <button class="btn-primary" :disabled="!auth.user || !channelStore.profile || cloud.loading" @click="saveCurrentProfile">
          Save current profile
        </button>
      </div>
      <p v-if="cloud.lastSavedAt">Last saved: {{ new Date(cloud.lastSavedAt).toLocaleString() }}</p>
      <ul>
        <li v-for="p in cloud.profiles" :key="p.id">
          {{ p.name }} — {{ new Date(p.updated_at).toLocaleString() }}
          <button class="btn-sm" @click="loadProfile(p)">Load</button>
        </li>
        <li v-if="auth.user && !cloud.profiles.length">No cloud profiles yet.</li>
      </ul>
      <p v-if="cloud.error" class="banner banner-error">{{ cloud.error }}</p>
    </section>

    <section>
      <h2>AI Credit</h2>
      <button class="btn" :disabled="!auth.user || creditLoading" @click="loadCredit">
        {{ creditLoading ? 'Checking...' : 'Check balance' }}
      </button>
      <p v-if="creditBalance !== null">Balance: {{ creditBalance }}</p>
      <p v-if="creditError" class="banner banner-error">{{ creditError }}</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { functionUrl } from '../api/supabase'
import { useAuthStore } from '../store/auth'
import { useChannelStore } from '../store/channels'
import { useCloudProfileStore } from '../store/cloudProfiles'
import type { CloudProfileRow } from '../types/cloud'

const auth = useAuthStore()
const channelStore = useChannelStore()
const cloud = useCloudProfileStore()

const email = ref('')
const password = ref('')
const creditBalance = ref<number | null>(null)
const creditLoading = ref(false)
const creditError = ref('')

async function signIn() {
  await auth.signIn(email.value, password.value)
  await cloud.loadProfiles()
}

async function signUp() {
  await auth.signUp(email.value, password.value)
  await cloud.loadProfiles()
}

async function saveCurrentProfile() {
  if (!channelStore.profile) return
  await cloud.saveProfile(channelStore.profile)
  channelStore.markClean()
}

function loadProfile(row: CloudProfileRow) {
  channelStore.setProfile(cloud.toProfile(row))
}

async function loadCredit() {
  if (!auth.session) return
  creditLoading.value = true
  creditError.value = ''
  try {
    const res = await fetch(functionUrl('ai-credit'), {
      headers: { Authorization: `Bearer ${auth.session.access_token}` },
    })
    const body = await res.json()
    if (!res.ok) throw new Error(body.error ?? 'Failed to query AI credit.')
    creditBalance.value = body.balance
  } catch (e) {
    creditError.value = e instanceof Error ? e.message : String(e)
  } finally {
    creditLoading.value = false
  }
}
</script>
