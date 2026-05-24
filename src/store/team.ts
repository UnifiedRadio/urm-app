import { defineStore } from 'pinia'
import { ref } from 'vue'
import { requireSupabase } from '../api/supabase'
import { useAuthStore } from './auth'
import type { ActivityRow, OrganizationRow } from '../types/cloud'

export const useTeamStore = defineStore('team', () => {
  const organizations = ref<OrganizationRow[]>([])
  const activities = ref<ActivityRow[]>([])
  const loading = ref(false)
  const error = ref('')

  async function loadAll() {
    const client = requireSupabase()
    loading.value = true
    error.value = ''
    try {
      const [{ data: orgs, error: orgError }, { data: acts, error: actError }] = await Promise.all([
        client.from('organizations').select('*').order('created_at', { ascending: false }),
        client.from('activities').select('*').order('created_at', { ascending: false }),
      ])
      if (orgError) throw orgError
      if (actError) throw actError
      organizations.value = (orgs ?? []) as OrganizationRow[]
      activities.value = (acts ?? []) as ActivityRow[]
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      loading.value = false
    }
  }

  async function createOrganization(name: string, description = '') {
    const auth = useAuthStore()
    if (!auth.user) throw new Error('Sign in before creating teams.')
    const client = requireSupabase()
    const inviteCode = Math.random().toString(36).slice(2, 10)
    const { data, error: insertError } = await client
      .from('organizations')
      .insert({
        name,
        description,
        owner_id: auth.user.id,
        invite_code: inviteCode,
        settings: {},
      })
      .select('*')
      .single()
    if (insertError) throw insertError
    await client.from('org_members').insert({ org_id: data.id, user_id: auth.user.id, role: 'owner' })
    await loadAll()
    return data as OrganizationRow
  }

  async function createActivity(input: {
    name: string
    orgId?: string | null
    description?: string
    deviceModel?: string
    profileId?: string | null
  }) {
    const auth = useAuthStore()
    if (!auth.user) throw new Error('Sign in before creating activities.')
    const client = requireSupabase()
    const { data, error: insertError } = await client
      .from('activities')
      .insert({
        name: input.name,
        org_id: input.orgId || null,
        creator_id: auth.user.id,
        description: input.description ?? '',
        device_model: input.deviceModel || null,
        status: 'planning',
      })
      .select('*')
      .single()
    if (insertError) throw insertError
    await client.from('activity_members').insert({
      activity_id: data.id,
      user_id: auth.user.id,
      role: 'owner',
      device_model: input.deviceModel || null,
    })
    if (input.profileId) {
      await client.from('activity_profiles').insert({ activity_id: data.id, profile_id: input.profileId })
    }
    await loadAll()
    return data as ActivityRow
  }

  return { organizations, activities, loading, error, loadAll, createOrganization, createActivity }
})
