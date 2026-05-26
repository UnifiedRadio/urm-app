<template>
  <div class="page page-dashboard">
    <h1>Dashboard</h1>

    <div class="dash-grid">
      <!-- Current profile card -->
      <div class="dash-card">
        <h3>当前配置</h3>
        <div v-if="channelStore.profile" class="stat-group">
          <div class="stat">
            <span class="stat-value">{{ channelStore.channelCount }}</span>
            <span class="stat-label">个频道</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ analogCount }}</span>
            <span class="stat-label">模拟 FM</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ dmrCount }}</span>
            <span class="stat-label">DMR</span>
          </div>
        </div>
        <p v-else class="hint">未加载配置文件。</p>
        <p v-if="channelStore.profile" class="profile-name">{{ channelStore.profile.name }}</p>
        <p v-if="channelStore.profile?.devices[0]" class="hint">
          设备：{{ channelStore.profile.devices[0].model }}
        </p>
        <button
          v-if="channelStore.isDirty"
          class="btn-warn"
          style="margin-top:.5rem"
          @click="ui.navigate('channel-editor')"
        >有未保存更改</button>
      </div>

      <!-- Cloud sync card -->
      <div class="dash-card">
        <h3>云端同步</h3>
        <div v-if="auth.user">
          <p class="hint">{{ auth.user.email }}</p>
          <div v-if="cloud.lastSavedAt" class="stat-group">
            <div class="stat">
              <span class="stat-value">{{ cloud.profiles.length }}</span>
              <span class="stat-label">云端配置</span>
            </div>
          </div>
          <p v-if="cloud.lastSavedAt" class="hint">
            上次同步：{{ new Date(cloud.lastSavedAt).toLocaleString('zh-CN') }}
          </p>
          <p v-else class="hint">尚未同步到云端。</p>
          <button class="btn btn-sm" style="margin-top:.5rem" @click="cloud.loadProfiles()">刷新</button>
        </div>
        <div v-else>
          <p class="hint">未登录，云端同步不可用。</p>
          <button class="btn btn-sm" @click="ui.navigate('settings')">去登录</button>
        </div>
      </div>

      <!-- Backup card -->
      <div class="dash-card">
        <h3>本地备份</h3>
        <div v-if="backups.length">
          <div class="stat-group">
            <div class="stat">
              <span class="stat-value">{{ backups.length }}</span>
              <span class="stat-label">条记录</span>
            </div>
          </div>
          <ul class="recent-list">
            <li v-for="b in recentBackups" :key="b.id">
              <span class="backup-kind-badge">{{ b.kind === 'profile_snapshot' ? '快照' : '设备' }}</span>
              <span>{{ b.device_model }}</span>
              <span class="hint">{{ fmtDate(b.created_at) }}</span>
            </li>
          </ul>
        </div>
        <p v-else class="hint">暂无备份。</p>
        <button class="btn btn-sm" style="margin-top:.5rem" @click="loadBackups">刷新</button>
      </div>

      <!-- Activities card -->
      <div class="dash-card">
        <h3>活动</h3>
        <div v-if="auth.user">
          <div class="stat-group">
            <div class="stat">
              <span class="stat-value">{{ team.activities.length }}</span>
              <span class="stat-label">个活动</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ team.organizations.length }}</span>
              <span class="stat-label">个团队</span>
            </div>
          </div>
          <ul v-if="team.activities.length" class="recent-list">
            <li v-for="a in team.activities.slice(0, 3)" :key="a.id">
              <span>{{ a.name }}</span>
              <span class="hint">{{ a.status }}</span>
            </li>
          </ul>
          <button class="btn btn-sm" style="margin-top:.5rem" @click="ui.navigate('team')">
            管理活动
          </button>
        </div>
        <div v-else>
          <p class="hint">登录后查看团队活动。</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useChannelStore } from '../store/channels'
import { useAuthStore } from '../store/auth'
import { useCloudProfileStore } from '../store/cloudProfiles'
import { useTeamStore } from '../store/team'
import { useUiStore } from '../store/ui'
import { listBackups } from '../api/config'
import { ref } from 'vue'
import type { BackupMeta } from '../types/urc'

const channelStore = useChannelStore()
const auth = useAuthStore()
const cloud = useCloudProfileStore()
const team = useTeamStore()
const ui = useUiStore()

const backups = ref<BackupMeta[]>([])
const recentBackups = computed(() => backups.value.slice(0, 5))

const analogCount = computed(
  () => channelStore.profile?.channels.filter(c => c.mode === 'analog_fm').length ?? 0,
)
const dmrCount = computed(
  () => channelStore.profile?.channels.filter(c => c.mode === 'dmr').length ?? 0,
)

async function loadBackups() {
  try { backups.value = await listBackups() } catch { /* ignore */ }
}

onMounted(async () => {
  loadBackups()
  if (auth.user) {
    cloud.loadProfiles()
    team.loadAll()
  }
})

function fmtDate(iso: string) {
  return new Date(iso).toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}
</script>

<style scoped>
.dash-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 1rem;
  margin-top: .5rem;
}

.dash-card {
  background: var(--color-surface, #1e1e1e);
  border: 1px solid var(--color-border, #333);
  border-radius: 8px;
  padding: 1rem;
}

.dash-card h3 {
  margin: 0 0 .75rem;
  font-size: .9rem;
  color: var(--color-muted, #888);
  text-transform: uppercase;
  letter-spacing: .05em;
}

.stat-group {
  display: flex;
  gap: 1.5rem;
  margin-bottom: .5rem;
}

.stat { display: flex; flex-direction: column; }
.stat-value { font-size: 1.8rem; font-weight: 700; line-height: 1; }
.stat-label { font-size: .75rem; color: var(--color-muted, #888); }

.profile-name { font-weight: 600; margin: .3rem 0 0; }

.recent-list {
  list-style: none;
  padding: 0;
  margin: .5rem 0 0;
  display: flex;
  flex-direction: column;
  gap: .3rem;
}
.recent-list li {
  display: flex;
  gap: .5rem;
  align-items: center;
  font-size: .85rem;
}

.backup-kind-badge {
  background: var(--color-bg, #111);
  border: 1px solid var(--color-border, #333);
  border-radius: 3px;
  padding: .1rem .3rem;
  font-size: .75rem;
}

.hint { font-size: .8rem; color: var(--color-muted, #888); margin: .2rem 0; }
</style>
