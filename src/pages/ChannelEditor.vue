<template>
  <div class="page page-channel-editor">

    <!-- ── Empty state / onboarding ── -->
    <div v-if="!store.profile" class="empty-state-onboarding">
      <h2>还没有配置文件</h2>
      <p>选择你的对讲机型号快速开始，或导入已有的 CHIRP CSV / URC 文件。</p>

      <div class="quick-templates">
        <button
          v-for="tpl in quickTemplates"
          :key="tpl.id"
          class="quick-tpl-btn"
          @click="applyTemplate(tpl)"
        >
          <span class="tpl-name">{{ tpl.name }}</span>
          <span class="tpl-range">{{ tpl.freq_ranges.map(r => r.label).join(' + ') }}</span>
        </button>
        <button class="quick-tpl-btn btn-outline" @click="ui.navigate('templates')">
          更多机型 →
        </button>
      </div>

      <div class="or-divider">或</div>

      <div class="import-shortcuts">
        <button class="btn" @click="importFile('chirp_csv')">导入 CHIRP CSV</button>
        <button class="btn" @click="importFile('urc_json')">导入 URC JSON</button>
        <button class="btn" @click="importFile('urc_yaml')">导入 URC YAML</button>
      </div>
    </div>

    <!-- ── Main editor (profile loaded) ── -->
    <template v-else>
      <!-- Toolbar -->
      <div class="toolbar">
        <div class="toolbar-left">
          <div class="dropdown" ref="importMenu">
            <button class="btn" @click="importMenuOpen = !importMenuOpen">导入 ▾</button>
            <div v-if="importMenuOpen" class="dropdown-list">
              <button @click="importFile('chirp_csv')">CHIRP CSV / .img</button>
              <button @click="importFile('urc_json')">URC JSON</button>
              <button @click="importFile('urc_yaml')">URC YAML</button>
            </div>
          </div>
          <div class="dropdown">
            <button class="btn" @click="exportMenuOpen = !exportMenuOpen">导出 ▾</button>
            <div v-if="exportMenuOpen" class="dropdown-list">
              <button @click="exportFile('chirp_csv')">CHIRP CSV（写频用）</button>
              <button @click="exportFile('urc_json')">URC JSON</button>
              <button @click="exportFile('urc_yaml')">URC YAML</button>
            </div>
          </div>
        </div>

        <div class="toolbar-center">
          <span class="profile-name">{{ store.profile.name }}</span>
          <span class="channel-count dim">{{ store.channelCount }} 个频道</span>
          <span v-if="boundModel" class="device-badge">{{ boundModelName }}</span>
        </div>

        <div class="toolbar-right">
          <button class="btn-accent" @click="openAddModal">+ 添加频道</button>
          <button class="btn" @click="runValidate" :disabled="validating">
            {{ validating ? '校验中…' : '校验' }}
          </button>
          <button class="btn-warn" v-if="store.isDirty" @click="saveSnapshot">
            保存快照
          </button>
          <button class="btn-ghost" @click="historyOpen = !historyOpen">
            历史 {{ historyOpen ? '▲' : '▼' }}
          </button>
          <button class="btn-ghost" @click="aiOpen = !aiOpen">AI 建议</button>
        </div>
      </div>

      <!-- AI assistant panel -->
      <div v-if="aiOpen" class="ai-panel">
        <div class="ai-panel-header">
          <span>AI 配置助手</span>
          <button class="btn-sm" @click="aiOpen = false">×</button>
        </div>
        <div class="ai-messages">
          <div v-for="(msg, i) in aiMessages" :key="i" :class="['ai-msg', `ai-msg-${msg.role}`]">
            <pre class="ai-msg-text">{{ msg.content }}</pre>
          </div>
          <div v-if="aiLoading" class="ai-msg ai-msg-assistant"><span class="dim">思考中…</span></div>
        </div>
        <div class="ai-input-row">
          <textarea
            v-model="aiPrompt"
            rows="2"
            placeholder="描述你需要的频道配置，例如：帮我添加本地 UHF 模拟中继信道…"
            @keydown.ctrl.enter="sendAi"
          />
          <button class="btn-primary" :disabled="aiLoading || !aiPrompt.trim()" @click="sendAi">发送</button>
        </div>
        <p v-if="aiError" class="banner banner-error">{{ aiError }}</p>
        <p v-if="!aiConfigured" class="banner banner-warn">
          请在设置中配置 AI API Key。
        </p>
      </div>

      <!-- Validation banners -->
      <div v-if="store.validationErrors.length" class="banner banner-error">
        <strong>校验错误：</strong>
        <ul>
          <li v-for="e in store.validationErrors" :key="e.message">{{ e.message }}</li>
        </ul>
      </div>
      <div v-if="validationWarnings.length" class="banner banner-warn">
        <strong>警告：</strong>
        <ul>
          <li v-for="w in validationWarnings" :key="w.message">{{ w.message }}</li>
        </ul>
      </div>

      <!-- Version history panel -->
      <div v-if="historyOpen" class="history-panel">
        <div class="history-panel-header">
          <span>版本历史</span>
          <button class="btn-sm" @click="loadBackups">刷新</button>
        </div>
        <div v-if="backupsLoading" class="dim">加载中…</div>
        <ul v-else-if="backups.length" class="backup-entries">
          <li v-for="b in backups" :key="b.id" class="backup-entry">
            <span class="backup-kind">{{ b.kind === 'profile_snapshot' ? '快照' : '设备备份' }}</span>
            <span class="backup-time">{{ fmt(b.created_at) }}</span>
            <span class="backup-size dim">{{ fmtSize(b.size_bytes) }}</span>
            <button
              v-if="b.kind === 'profile_snapshot'"
              class="btn-sm"
              @click="restoreFrom(b.id)"
            >还原</button>
          </li>
        </ul>
        <div v-else class="dim">暂无快照。编辑后点击"保存快照"。</div>
      </div>

      <!-- Channel table -->
      <ChannelTable
        :profile="store.profile"
        :bound-model="boundModel"
        @edit="openEditModal"
        @delete="deleteChannel"
      />
    </template>

    <!-- ── Channel add/edit modal ── -->
    <div v-if="modalOpen" class="modal-backdrop" @click.self="modalOpen = false">
      <div class="modal">
        <h2>{{ editingChannel ? '编辑频道' : '添加频道' }}</h2>
        <form @submit.prevent="submitModal">

          <label>频道名称
            <input
              v-model="form.name"
              :maxlength="boundModelMaxName"
              required
              placeholder="最多 {{ boundModelMaxName }} 个字符"
            />
            <span v-if="boundModelMaxName < 16" class="field-hint">
              {{ form.name.length }}/{{ boundModelMaxName }}（{{ boundModelName }} 硬件限制）
            </span>
          </label>

          <label>模式
            <select v-model="form.mode">
              <option value="analog_fm">模拟 FM</option>
              <option value="dmr">DMR</option>
            </select>
          </label>

          <div class="freq-row">
            <label>接收频率 (MHz)
              <input
                type="number" v-model.number="form.rx_freq_mhz"
                step="0.0001" min="0" required
                :class="{ invalid: !rxFreqValid }"
              />
              <span v-if="boundModel && !rxFreqValid" class="field-error">
                超出 {{ boundModelName }} 支持范围
              </span>
            </label>
            <label>发射频率 (MHz)
              <input
                type="number" v-model.number="form.tx_freq_mhz"
                step="0.0001" min="0" required
              />
              <button type="button" class="btn-sm" @click="form.tx_freq_mhz = form.rx_freq_mhz">
                ← 同接收
              </button>
            </label>
          </div>

          <!-- Freq range hint for bound device -->
          <div v-if="boundModel && freqRangeHint" class="field-hint freq-hint">
            {{ boundModelName }} 支持频段：{{ freqRangeHint }}
          </div>

          <div class="form-row">
            <label>功率
              <select v-model="form.power">
                <option value="high">高（High）</option>
                <option value="medium">中（Medium）</option>
                <option value="low">低（Low）</option>
              </select>
            </label>
            <label>带宽
              <select v-model="form.bandwidth">
                <option value="narrow">窄（12.5 kHz）</option>
                <option value="wide">宽（25 kHz）</option>
              </select>
            </label>
          </div>

          <label>标签（逗号分隔）
            <input v-model="tagsInput" placeholder="中继, 应急, simplex" />
          </label>

          <AnalogForm v-if="form.mode === 'analog_fm'" v-model="form.analog" />
          <DmrForm v-else-if="form.mode === 'dmr'" v-model="form.dmr" />

          <div class="modal-actions">
            <button type="button" class="btn" @click="modalOpen = false">取消</button>
            <button type="submit" class="btn-primary">{{ editingChannel ? '保存' : '添加' }}</button>
          </div>
        </form>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch, onMounted, onUnmounted } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useChannelStore } from '../store/channels'
import { useUiStore } from '../store/ui'
import {
  importChirpCsv, importUrcJson, importUrcYaml,
  exportChirpCsv, exportUrcJson, exportUrcYaml,
  validateProfile, validateForDevice, saveProfileSnapshot,
  listBackups, restoreBackup,
} from '../api/config'
import ChannelTable from '../components/ChannelEditor/ChannelTable.vue'
import AnalogForm from '../components/ChannelEditor/AnalogForm.vue'
import DmrForm from '../components/ChannelEditor/DmrForm.vue'
import {
  DEVICE_TEMPLATES, freqRangesForModel, isFreqValid,
  type DeviceTemplate,
} from '../data/deviceTemplates'
import type { Channel, AnalogFields, DmrFields, BackupMeta } from '../types/urc'
import { useAiConfigStore } from '../store/aiConfig'
import { callAi } from '../api/ai'

const store = useChannelStore()
const ui = useUiStore()

// ── Onboarding shortcuts ─────────────────────────────────────────────────────
const quickTemplates = DEVICE_TEMPLATES.filter(t => t.category === 'beginner')

function applyTemplate(tpl: DeviceTemplate) {
  store.setProfile({
    ...tpl.starter_profile,
    id: crypto.randomUUID(),
    name: `${tpl.name} – ${new Date().toLocaleDateString('zh-CN')}`,
    channels: [],
  })
}

// ── Device binding helpers ───────────────────────────────────────────────────
const boundModel = computed(() => store.profile?.devices[0]?.model ?? '')
const boundModelName = computed(() => {
  const tpl = DEVICE_TEMPLATES.find(t => t.model === boundModel.value)
  return tpl?.name ?? boundModel.value
})
const boundModelMaxName = computed(() => {
  const tpl = DEVICE_TEMPLATES.find(t => t.model === boundModel.value)
  return tpl?.max_name_chars ?? 16
})
const freqRangeHint = computed(() => {
  if (!boundModel.value) return ''
  return freqRangesForModel(boundModel.value)
    .map(r => `${r.label} ${r.min_mhz}–${r.max_mhz} MHz`)
    .join('，')
})

// ── Toolbar state ────────────────────────────────────────────────────────────
const importMenuOpen = ref(false)
const exportMenuOpen = ref(false)
const validating = ref(false)
const validationWarnings = ref<import('../types/urc').ValidationItem[]>([])

// ── History panel ────────────────────────────────────────────────────────────
const historyOpen = ref(false)
const backups = ref<BackupMeta[]>([])
const backupsLoading = ref(false)

async function loadBackups() {
  backupsLoading.value = true
  try { backups.value = await listBackups() }
  catch { /* silently ignore if dir not yet created */ }
  finally { backupsLoading.value = false }
}

async function restoreFrom(id: string) {
  if (!confirm('将用该快照覆盖当前编辑内容，确定还原？')) return
  try {
    const profile = await restoreBackup(id)
    store.setProfile(profile)
    historyOpen.value = false
  } catch (e) {
    alert(`还原失败：${e}`)
  }
}

// Auto-load backups when panel opens
watch(historyOpen, open => {
  if (open && !backups.value.length) loadBackups()
})

// ── Import / Export ──────────────────────────────────────────────────────────
async function importFile(format: 'urc_json' | 'urc_yaml' | 'chirp_csv') {
  importMenuOpen.value = false
  const filters =
    format === 'chirp_csv'
      ? [{ name: 'CHIRP CSV / IMG', extensions: ['csv', 'img', 'CSV'] }]
      : format === 'urc_yaml'
      ? [{ name: 'URC YAML', extensions: ['yaml', 'yml'] }]
      : [{ name: 'URC JSON', extensions: ['json'] }]

  const selected = await open({ multiple: false, filters })
  if (!selected || typeof selected !== 'string') return

  try {
    const profile =
      format === 'chirp_csv' ? await importChirpCsv(selected)
      : format === 'urc_yaml' ? await importUrcYaml(selected)
      : await importUrcJson(selected)
    store.setProfile(profile)
    // Auto-validate against bound device after import
    if (profile.devices[0]?.model) {
      await runValidateForDevice(profile.devices[0].model)
    }
  } catch (e) {
    alert(`导入失败：${e}`)
  }
}

async function exportFile(format: 'urc_json' | 'urc_yaml' | 'chirp_csv') {
  exportMenuOpen.value = false
  if (!store.profile) return
  const ext = format === 'chirp_csv' ? 'csv' : format === 'urc_yaml' ? 'yaml' : 'json'
  const defaultName = `${store.profile.name.replace(/[/\\:*?"<>|]/g, '_')}.${ext}`
  const path = await save({ defaultPath: defaultName })
  if (!path) return
  try {
    if (format === 'chirp_csv') await exportChirpCsv(store.profile, path)
    else if (format === 'urc_yaml') await exportUrcYaml(store.profile, path)
    else await exportUrcJson(store.profile, path)
  } catch (e) {
    alert(`导出失败：${e}`)
  }
}

// ── Validation ───────────────────────────────────────────────────────────────
async function runValidate() {
  if (!store.profile) return
  validating.value = true
  try {
    const model = boundModel.value
    const result = model
      ? await validateForDevice(store.profile, model)
      : await validateProfile(store.profile)
    store.validationErrors.splice(0, store.validationErrors.length, ...result.errors)
    validationWarnings.value = result.warnings
  } finally {
    validating.value = false
  }
}

async function runValidateForDevice(model: string) {
  if (!store.profile) return
  const result = await validateForDevice(store.profile, model)
  store.validationErrors.splice(0, store.validationErrors.length, ...result.errors)
  validationWarnings.value = result.warnings
}

// ── Snapshot ─────────────────────────────────────────────────────────────────
async function saveSnapshot() {
  if (!store.profile) return
  try {
    await saveProfileSnapshot(store.profile)
    store.markClean()
    if (historyOpen.value) loadBackups()
  } catch (e) {
    alert(`快照保存失败：${e}`)
  }
}

// ── Channel CRUD ─────────────────────────────────────────────────────────────
type ChannelForm = {
  name: string
  mode: Channel['mode']
  rx_freq_mhz: number
  tx_freq_mhz: number
  power: Channel['power']
  bandwidth: Channel['bandwidth']
  analog: AnalogFields
  dmr: DmrFields
}

const defaultAnalog = (): AnalogFields => ({ tone_type: 'none', scan: true })
const defaultDmr = (): DmrFields => ({
  color_code: 1,
  time_slot: 1,
  talkgroup_id: 1,
  contact_ref: '',
  rx_group_ref: '',
  zone_ref: '',
})
const defaultFreq = () => {
  // Pre-fill with first valid frequency for the bound device
  const ranges = boundModel.value ? freqRangesForModel(boundModel.value) : []
  if (ranges.length > 0) {
    const mid = ((ranges[0].min_mhz + ranges[0].max_mhz) / 2)
    return Math.round(mid * 10) / 10
  }
  return 145.5
}

const form = reactive<ChannelForm>({
  name: '',
  mode: 'analog_fm',
  rx_freq_mhz: 145.5,
  tx_freq_mhz: 145.5,
  power: 'high',
  bandwidth: 'narrow',
  analog: defaultAnalog(),
  dmr: defaultDmr(),
})
const tagsInput = ref('')
const modalOpen = ref(false)
const editingChannel = ref<Channel | null>(null)

const rxFreqValid = computed(() => {
  if (!boundModel.value || form.rx_freq_mhz <= 0) return true
  return isFreqValid(form.rx_freq_mhz, boundModel.value)
})

function openAddModal() {
  editingChannel.value = null
  const f = defaultFreq()
  Object.assign(form, {
    name: '',
    mode: 'analog_fm',
    rx_freq_mhz: f,
    tx_freq_mhz: f,
    power: 'high',
    bandwidth: 'narrow',
    analog: defaultAnalog(),
    dmr: defaultDmr(),
  })
  tagsInput.value = ''
  modalOpen.value = true
}

function openEditModal(ch: Channel) {
  editingChannel.value = ch
  Object.assign(form, {
    name: ch.name,
    mode: ch.mode,
    rx_freq_mhz: ch.rx_freq_mhz,
    tx_freq_mhz: ch.tx_freq_mhz,
    power: ch.power,
    bandwidth: ch.bandwidth,
    analog: ch.analog ? { ...ch.analog } : defaultAnalog(),
    dmr: ch.dmr ? { ...ch.dmr } : defaultDmr(),
  })
  tagsInput.value = ch.tags.join(', ')
  modalOpen.value = true
}

function submitModal() {
  const tags = tagsInput.value.split(',').map(t => t.trim()).filter(Boolean)
  const base = {
    name: form.name.slice(0, boundModelMaxName.value),
    mode: form.mode,
    rx_freq_mhz: form.rx_freq_mhz,
    tx_freq_mhz: form.tx_freq_mhz,
    power: form.power,
    bandwidth: form.bandwidth,
    tags,
    analog: form.mode === 'analog_fm' ? { ...form.analog } : undefined,
    dmr: form.mode === 'dmr' ? { ...form.dmr } : undefined,
  }
  if (editingChannel.value) {
    store.updateChannel({ ...editingChannel.value, ...base })
  } else {
    store.addChannel({ id: crypto.randomUUID(), ...base })
  }
  modalOpen.value = false
}

function deleteChannel(ch: Channel) {
  if (!confirm(`删除频道"${ch.name}"？`)) return
  store.removeChannel(ch.id)
}

// ── AI assistant ─────────────────────────────────────────────────────────────
const aiConfig = useAiConfigStore()
const aiOpen = ref(false)
const aiPrompt = ref('')
const aiMessages = ref<{ role: 'user' | 'assistant'; content: string }[]>([])
const aiLoading = ref(false)
const aiError = ref('')
const aiConfigured = computed(() => Boolean(aiConfig.apiKey))

async function sendAi() {
  const prompt = aiPrompt.value.trim()
  if (!prompt || aiLoading.value) return
  aiMessages.value.push({ role: 'user', content: prompt })
  aiPrompt.value = ''
  aiLoading.value = true
  aiError.value = ''
  try {
    const context = store.profile
      ? `当前配置 "${store.profile.name}" 有 ${store.profile.channels.length} 个频道。设备：${store.profile.devices[0]?.model ?? '未绑定'}。`
      : '当前无配置文件。'
    const reply = await callAi(`${context}\n\n${prompt}`, aiConfig)
    aiMessages.value.push({ role: 'assistant', content: reply })
  } catch (e) {
    aiError.value = e instanceof Error ? e.message : String(e)
  } finally {
    aiLoading.value = false
  }
}

// ── Formatting helpers ───────────────────────────────────────────────────────
function fmt(iso: string) {
  return new Date(iso).toLocaleString('zh-CN')
}
function fmtSize(bytes: number) {
  return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(1)} KB`
}
</script>
