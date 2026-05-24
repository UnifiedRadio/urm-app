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
        <li v-for="activity in team.activities" :key="activity.id">
          {{ activity.name }} — {{ activity.status }} <span v-if="activity.device_model">· {{ activity.device_model }}</span>
        </li>
        <li v-if="auth.user && !team.activities.length">No activities yet.</li>
      </ul>
    </section>

    <p v-if="team.error" class="banner banner-error">{{ team.error }}</p>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useAuthStore } from '../store/auth'
import { useChannelStore } from '../store/channels'
import { useCloudProfileStore } from '../store/cloudProfiles'
import { useTeamStore } from '../store/team'

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

onMounted(() => {
  if (auth.user) team.loadAll()
})

watch(() => auth.user?.id, id => {
  if (id) team.loadAll()
})

async function createTeam() {
  await team.createOrganization(teamName.value, teamDescription.value)
  teamName.value = ''
  teamDescription.value = ''
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
