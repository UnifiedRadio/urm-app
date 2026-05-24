import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { Session, User } from '@supabase/supabase-js'
import { isCloudConfigured, requireSupabase, supabase } from '../api/supabase'
import { useAsync } from '../composables/useAsync'

export const useAuthStore = defineStore('auth', () => {
  const session = ref<Session | null>(null)
  const user = computed<User | null>(() => session.value?.user ?? null)
  const initialized = ref(false)
  const { loading, error, run } = useAsync()

  async function init() {
    if (initialized.value) return
    initialized.value = true
    if (!supabase) return
    const { data } = await supabase.auth.getSession()
    session.value = data.session
    if (data.session?.user) await ensureUserRecord(data.session.user)
    supabase.auth.onAuthStateChange(async (_event, nextSession) => {
      session.value = nextSession
      if (nextSession?.user) await ensureUserRecord(nextSession.user)
    })
  }

  async function ensureUserRecord(currentUser = user.value) {
    if (!currentUser) return
    const client = requireSupabase()
    await client.from('users').upsert({
      id: currentUser.id,
      email: currentUser.email ?? `${currentUser.id}@local.openradio`,
    })
  }

  async function signIn(email: string, password: string) {
    await run(async () => {
      const client = requireSupabase()
      const { data, error: signInError } = await client.auth.signInWithPassword({ email, password })
      if (signInError) throw signInError
      session.value = data.session
      await ensureUserRecord(data.user)
    })
  }

  async function signUp(email: string, password: string) {
    await run(async () => {
      const client = requireSupabase()
      const { data, error: signUpError } = await client.auth.signUp({ email, password })
      if (signUpError) throw signUpError
      session.value = data.session
      if (data.user) await ensureUserRecord(data.user)
    })
  }

  async function signOut() {
    const client = requireSupabase()
    await client.auth.signOut()
    session.value = null
  }

  return { session, user, loading, error, initialized, isCloudConfigured, init, signIn, signUp, signOut, ensureUserRecord }
})
