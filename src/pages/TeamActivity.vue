<template>
  <div class="page page-team">
    <h1>Team / Activity</h1>
    <p v-if="!auth.user" class="banner banner-warn">Sign in before creating teams or activities.</p>

    <section>
      <h2>Create Team</h2>
      <form @submit.prevent="createTeam">
        <label>Name
          <input v-model="teamName" required />
        </label>
        <label>Description
          <textarea v-model="teamDescription" />
        </label>
        <button class="btn-primary" :disabled="!auth.user || team.loading">Create team</button>
      </form>
    </section>

    <section>
      <h2>Create Activity</h2>
      <form @submit.prevent="createActivity">
        <label>Name
          <input v-model="activityName" required />
        </label>
        <label>Team
          <select v-model="activityOrgId">
            <option value="">Personal activity</option>
            <option v-for="org in team.organizations" :key="org.id" :value="org.id">{{ org.name }}</option>
          </select>
        </label>
        <label>Device model
          <input v-model="activityDeviceModel" placeholder="baofeng_uv5r" />
        </label>
        <label>
          <input v-model="bindCurrentProfile" type="checkbox" />
          Bind current profile
        </label>
        <button class="btn-primary" :disabled="!auth.user || team.loading">Create activity</button>
      </form>
    </section>

    <section>
      <h2>加入团队</h2>
      <form @submit.prevent="joinTeam">
        <label>邀请码
          <input v-model="inviteCode" placeholder="8位邀请码" maxlength="8" required />
        </label>
        <button class="btn-primary" :disabled="!auth.user || team.loading || !inviteCode">加入</button>
      </form>
      <p v-if="joinError" class="banner banner-error">{{ joinError }}</p>
      <p v-if="joinSuccess" class="banner banner-ok">{{ joinSuccess }}</p>
    </section>

    <section>
      <h2>Teams</h2>
      <button class="btn" :disabled="!auth.user || team.loading" @click="team.loadAll()">Refresh</button>
      <ul>
        <li v-for="org in team.organizations" :key="org.id">
          {{ org.name }} <span v-if="org.invite_code">invite: {{ org.invite_code }}</span>
        </li>
        <li v-if="auth.user && !team.organizations.length">No teams yet.</li>
      </ul>
    </section>

    <section>
      <h2>Activities</h2>
      <ul>
        <li
          v-for="activity in team.activities"
          :key="activity.id"
          class="activity-item"
          :class="{ 'activity-selected': selectedActivityId === activity.id }"
          @click="selectActivity(activity.id)"
        >
          <span>{{ activity.name }} — {{ activity.status }}</span>
          <span v-if="activity.device_model" class="activity-model">· {{ activity.device_model }}</span>
        </li>
        <li v-if="auth.user && !team.activities.length">No activities yet.</li>
      </ul>
    </section>

    <!-- Risk Pack panel for selected activity -->
    <section v-if="selectedActivityId">
      <h2>Risk Pack — {{ selectedActivityName }}</h2>
      <RiskPackPanel :activity-id="selectedActivityId" />
    </section>

    <p v-if="team.error" class="banner banner-error">{{ team.error }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useAuthStore } from '../store/auth'
import { useChannelStore } from '../store/channels'
import { useCloudProfileStore } from '../store/cloudProfiles'
import { useTeamStore } from '../store/team'
import { requireSupabase } from '../api/supabase'
import RiskPackPanel from '../components/RiskPack/RiskPackPanel.vue'

const auth = useAuthStore()
const channelStore = useChannelStore()
const cloud = useCloudProfileStore()
const team = useTeamStore()

const teamName = ref('')
const teamDescription = ref('')
const activityName = ref('')
const activityOrgId = ref('')
const activityDeviceModel = ref('baofeng_uv5r')
const bindCurrentProfile = ref(true)
const selectedActivityId = ref<string | null>(null)
const inviteCode = ref('')
const joinError = ref('')
const joinSuccess = ref('')

const selectedActivityName = computed(
  () => team.activities.find(a => a.id === selectedActivityId.value)?.name ?? '',
)

onMounted(() => {
  if (auth.user) team.loadAll()
})

watch(() => auth.user?.id, id => {
  if (id) team.loadAll()
})

function selectActivity(id: string) {
  selectedActivityId.value = selectedActivityId.value === id ? null : id
}

async function createTeam() {
  await team.createOrganization(teamName.value, teamDescription.value)
  teamName.value = ''
  teamDescription.value = ''
}

async function joinTeam() {
  joinError.value = ''
  joinSuccess.value = ''
  try {
    const client = requireSupabase()
    const { data: org, error: orgErr } = await client
      .from('organizations')
      .select('id, name')
      .eq('invite_code', inviteCode.value.trim())
      .single()
    if (orgErr || !org) throw new Error('邀请码无效或已过期。')
    const { error: memberErr } = await client
      .from('org_members')
      .insert({ org_id: org.id, user_id: auth.user!.id, role: 'member' })
    if (memberErr && !memberErr.message.includes('duplicate')) throw memberErr
    joinSuccess.value = `成功加入团队"${org.name}"。`
    inviteCode.value = ''
    await team.loadAll()
  } catch (e) {
    joinError.value = e instanceof Error ? e.message : String(e)
  }
}

async function createActivity() {
  let profileId: string | null = null
  if (bindCurrentProfile.value && channelStore.profile) {
    const row = await cloud.saveProfile(channelStore.profile, activityOrgId.value || null)
    profileId = row.id
  }
  await team.createActivity({
    name: activityName.value,
    orgId: activityOrgId.value || null,
    deviceModel: activityDeviceModel.value,
    profileId,
  })
  activityName.value = ''
}
</script>

<style scoped>
.activity-item {
  cursor: pointer;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  display: flex;
  gap: 0.4rem;
  align-items: center;
}
.activity-item:hover { background: var(--color-surface-hover, #2a2a2a); }
.activity-selected { background: var(--color-surface-active, #1e3a5f) !important; }
.activity-model { color: var(--color-muted, #888); font-size: 0.85rem; }
</style>
