import { defineStore } from 'pinia'
import { ref } from 'vue'
import { functionUrl, requireSupabase } from '../api/supabase'
import { useAuthStore } from './auth'
import type { UrcProfile } from '../types/urc'
import type { CloudProfileRow, SharePackResponse, SharePreview } from '../types/cloud'

export const useCloudProfileStore = defineStore('cloudProfiles', () => {
  const profiles = ref<CloudProfileRow[]>([])
  const currentCloudId = ref<string | null>(null)
  const currentLocalProfileId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref('')
  const lastSavedAt = ref<string | null>(null)

  async function loadProfiles() {
    const client = requireSupabase()
    loading.value = true
    error.value = ''
    try {
      const { data, error: queryError } = await client
        .from('profiles')
        .select('*')
        .order('updated_at', { ascending: false })
      if (queryError) throw queryError
      profiles.value = (data ?? []) as CloudProfileRow[]
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function saveProfile(profile: UrcProfile, orgId: string | null = null) {
    const auth = useAuthStore()
    if (!auth.user) throw new Error('Sign in before saving cloud profiles.')
    const client = requireSupabase()
    loading.value = true
    error.value = ''
    try {
      const payload = {
          owner_id: auth.user.id,
          org_id: orgId,
          name: profile.name,
          urc_data: profile,
          updated_at: new Date().toISOString(),
        }
      const shouldUpdate = currentCloudId.value !== null && currentLocalProfileId.value === profile.id
      const query = shouldUpdate
        ? client.from('profiles').update(payload).eq('id', currentCloudId.value)
        : client.from('profiles').insert(payload)
      const { data, error: saveError } = await query
        .select('*')
        .single()
      if (saveError) throw saveError
      currentCloudId.value = data.id
      currentLocalProfileId.value = profile.id
      lastSavedAt.value = data.updated_at
      await loadProfiles()
      return data as CloudProfileRow
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  function toProfile(row: CloudProfileRow): UrcProfile {
    currentCloudId.value = row.id
    currentLocalProfileId.value = row.urc_data.id
    lastSavedAt.value = row.updated_at
    return row.urc_data
  }

  async function createShareLink(profileId: string, expiresInDays = 30) {
    const auth = useAuthStore()
    if (!auth.session) throw new Error('Sign in before creating share links.')
    const res = await fetch(functionUrl('share-pack'), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${auth.session.access_token}`,
      },
      body: JSON.stringify({ profile_id: profileId, expires_in_days: expiresInDays }),
    })
    const body = await res.json()
    if (!res.ok) throw new Error(body.error ?? 'Failed to create share link.')
    return body as SharePackResponse
  }

  async function fetchSharePreview(shortCode: string) {
    const res = await fetch(`${functionUrl('share-pack')}?code=${encodeURIComponent(shortCode)}`)
    const body = await res.json()
    if (!res.ok) throw new Error(body.error ?? 'Failed to load shared profile.')
    return body as SharePreview
  }

  async function importSharedProfile(preview: SharePreview) {
    const clone: UrcProfile = {
      ...preview.profile,
      id: crypto.randomUUID(),
      name: `${preview.profile.name} (imported)`,
      owner_type: 'personal',
    }
    await saveProfile(clone)
    return clone
  }

  return {
    profiles,
    currentCloudId,
    currentLocalProfileId,
    loading,
    error,
    lastSavedAt,
    loadProfiles,
    saveProfile,
    toProfile,
    createShareLink,
    fetchSharePreview,
    importSharedProfile,
  }
})
