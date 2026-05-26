<template>
  <div class="risk-pack-panel">
    <!-- Mandatory disclaimer — never dismissible or collapsible -->
    <div class="disclaimer-banner">
      请以官方信息为准。ORW 仅用于活动组织、通信协同与信息入口整理，不发布官方灾害预警，不替代政府、气象、交通、导航或应急广播系统。活动前请由队长再次确认天气、灾害、交通与当地官方通知。
    </div>

    <!-- Location & coordinates -->
    <section class="rp-section">
      <h3>活动地点</h3>
      <label>地点名称
        <input v-model="draft.location_name" placeholder="例：某山顶营地" />
      </label>
      <div class="coord-row">
        <label>纬度
          <input type="number" v-model.number="draft.latitude" step="0.0001" placeholder="35.6895" />
        </label>
        <label>经度
          <input type="number" v-model.number="draft.longitude" step="0.0001" placeholder="139.6917" />
        </label>
      </div>
    </section>

    <!-- Weather -->
    <section class="rp-section">
      <h3>天气预报
        <button
          class="btn btn-sm"
          :disabled="!canFetchWeather || weatherLoading"
          @click="refreshWeather"
        >{{ weatherLoading ? '加载中…' : '刷新' }}</button>
      </h3>
      <p v-if="!canFetchWeather" class="hint">请先填写经纬度以获取天气。</p>
      <p v-if="weatherError" class="banner banner-error">{{ weatherError }}</p>
      <div v-if="draft.weather_cache" class="weather-grid">
        <div
          v-for="h in visibleForecast"
          :key="h.time"
          class="weather-card"
        >
          <div class="wc-time">{{ formatHour(h.time) }}</div>
          <div class="wc-temp">{{ h.temperature_2m.toFixed(1) }}°C</div>
          <div class="wc-precip">{{ h.precipitation_probability }}%</div>
          <div class="wc-wind">{{ h.windspeed_10m.toFixed(1) }} km/h</div>
        </div>
      </div>
      <p v-if="draft.weather_cache" class="hint">
        数据来源：Open-Meteo，{{ formatFetchedAt(draft.weather_cache.fetched_at) }}
      </p>
    </section>

    <!-- Nearby repeaters -->
    <section class="rp-section">
      <RepeaterPanel :latitude="draft.latitude" :longitude="draft.longitude" />
    </section>

    <!-- Official links -->
    <section class="rp-section">
      <h3>官方信息链接</h3>
      <div v-for="(link, idx) in draft.official_links" :key="link.id" class="link-row">
        <input v-model="link.label" placeholder="链接名称" class="link-label" />
        <input v-model="link.url" placeholder="https://..." class="link-url" />
        <select v-model="link.category" class="link-cat">
          <option value="weather">天气</option>
          <option value="disaster">灾害</option>
          <option value="traffic">交通</option>
          <option value="local">当地</option>
          <option value="other">其他</option>
        </select>
        <button v-if="link.url" class="btn btn-sm" @click="openUrl(link.url)" title="在浏览器中打开">↗</button>
        <button class="btn btn-sm btn-danger" @click="removeLink(idx)">删除</button>
      </div>
      <button class="btn btn-sm" @click="addLink">+ 添加链接</button>
    </section>

    <!-- Checklist -->
    <section class="rp-section">
      <h3>出发前确认清单</h3>
      <div v-for="(item, idx) in draft.checklist" :key="item.id" class="checklist-row">
        <input type="checkbox" v-model="item.checked" :id="`chk-${item.id}`" />
        <label :for="`chk-${item.id}`" class="chk-label">{{ item.text }}</label>
        <button class="btn btn-sm btn-danger" @click="removeChecklistItem(idx)">×</button>
      </div>
      <div class="checklist-add">
        <input v-model="newChecklistText" placeholder="添加确认项…" @keydown.enter="addChecklistItem" />
        <button class="btn btn-sm" @click="addChecklistItem">+ 添加</button>
      </div>
    </section>

    <!-- Captain notes -->
    <section class="rp-section">
      <h3>队长备注</h3>
      <textarea v-model="draft.captain_notes" rows="4" placeholder="队长在此填写注意事项、补充说明…" />
    </section>

    <!-- AI summary -->
    <section class="rp-section ai-section">
      <h3>AI 风险摘要
        <button
          class="btn btn-sm"
          :disabled="aiSummaryLoading || !aiConfigured"
          @click="generateAiSummary"
        >{{ aiSummaryLoading ? '生成中…' : '生成摘要' }}</button>
      </h3>
      <p v-if="!aiConfigured" class="hint">请在设置中配置 AI API Key。</p>
      <p v-if="aiSummaryError" class="banner banner-error">{{ aiSummaryError }}</p>
      <div v-if="draft.ai_summary" class="ai-summary-box">
        <p>{{ draft.ai_summary.text }}</p>
        <p class="hint">生成时间：{{ draft.ai_summary.generated_at }}</p>
      </div>
      <div class="ai-disclaimer">AI 摘要仅供辅助理解，不作为安全决策依据。</div>
    </section>

    <!-- Actions -->
    <div class="rp-actions">
      <p v-if="riskPackStore.error" class="banner banner-error">{{ riskPackStore.error }}</p>
      <button class="btn-primary" :disabled="riskPackStore.loading" @click="save">
        {{ riskPackStore.loading ? '保存中…' : '保存 Risk Pack' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { open as shellOpen } from '@tauri-apps/plugin-shell'
import RepeaterPanel from './RepeaterPanel.vue'
import { useRiskPackStore } from '../../store/riskPack'
import { useAiConfigStore } from '../../store/aiConfig'
import { callAiRiskSummary } from '../../api/ai'
import type { RiskPack, RiskPackSource, RiskPackChecklistItem } from '../../types/cloud'

const props = defineProps<{ activityId: string }>()

const riskPackStore = useRiskPackStore()
const aiConfig = useAiConfigStore()

const aiConfigured = computed(() => Boolean(aiConfig.apiKey))
const aiSummaryLoading = ref(false)
const aiSummaryError = ref('')

const draft = ref<RiskPack>(riskPackStore.defaultRiskPack())
const weatherLoading = ref(false)
const weatherError = ref<string | null>(null)
const newChecklistText = ref('')

watch(
  () => riskPackStore.current,
  v => { if (v) draft.value = JSON.parse(JSON.stringify(v)) },
  { immediate: true },
)

watch(
  () => props.activityId,
  id => { if (id) riskPackStore.loadForActivity(id) },
  { immediate: true },
)

const canFetchWeather = computed(
  () => draft.value.latitude != null && draft.value.longitude != null,
)

const visibleForecast = computed(() => {
  const fc = draft.value.weather_cache?.forecast ?? []
  return fc.slice(0, 24)
})

async function refreshWeather() {
  if (!canFetchWeather.value) return
  weatherLoading.value = true
  weatherError.value = null
  try {
    const weather = await riskPackStore.fetchWeather(
      draft.value.latitude!,
      draft.value.longitude!,
    )
    draft.value = { ...draft.value, weather_cache: weather }
  } catch (e) {
    weatherError.value = (e as Error).message
  } finally {
    weatherLoading.value = false
  }
}

function addLink() {
  draft.value.official_links.push({
    id: crypto.randomUUID(),
    label: '',
    url: '',
    category: 'other',
  } as RiskPackSource)
}

function removeLink(idx: number) {
  draft.value.official_links.splice(idx, 1)
}

function addChecklistItem() {
  const text = newChecklistText.value.trim()
  if (!text) return
  draft.value.checklist.push({
    id: crypto.randomUUID(),
    text,
    checked: false,
  } as RiskPackChecklistItem)
  newChecklistText.value = ''
}

function removeChecklistItem(idx: number) {
  draft.value.checklist.splice(idx, 1)
}

async function openUrl(url: string) {
  try { await shellOpen(url) } catch { window.open(url, '_blank') }
}

async function generateAiSummary() {
  aiSummaryLoading.value = true
  aiSummaryError.value = ''
  try {
    const checklistItems = draft.value.checklist.map(
      c => `[${c.checked ? '✓' : ' '}] ${c.text}`,
    )
    const links = draft.value.official_links.map(l => l.url).filter(Boolean)
    const text = await callAiRiskSummary(
      draft.value.location_name,
      checklistItems,
      links,
      { baseUrl: aiConfig.baseUrl, apiKey: aiConfig.apiKey, model: aiConfig.model },
    )
    draft.value = {
      ...draft.value,
      ai_summary: { text, generated_at: new Date().toISOString() },
    }
  } catch (e) {
    aiSummaryError.value = e instanceof Error ? e.message : String(e)
  } finally {
    aiSummaryLoading.value = false
  }
}

async function save() {
  await riskPackStore.saveRiskPack(props.activityId, draft.value)
}

function formatHour(iso: string): string {
  const d = new Date(iso)
  return `${(d.getMonth() + 1)}/${d.getDate()} ${String(d.getHours()).padStart(2, '0')}:00`
}

function formatFetchedAt(iso: string): string {
  return new Date(iso).toLocaleString()
}
</script>

<style scoped>
.risk-pack-panel {
  display: flex;
  flex-direction: column;
  gap: 1.2rem;
}

.disclaimer-banner {
  background: #7c2d12;
  color: #fef3c7;
  border: 1px solid #b45309;
  border-radius: 6px;
  padding: 0.75rem 1rem;
  font-size: 0.85rem;
  line-height: 1.6;
}

.rp-section {
  background: var(--color-surface, #1e1e1e);
  border: 1px solid var(--color-border, #333);
  border-radius: 6px;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

.rp-section h3 {
  margin: 0 0 0.4rem;
  font-size: 0.95rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.coord-row {
  display: flex;
  gap: 1rem;
}

.coord-row label {
  flex: 1;
}

.weather-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  margin-top: 0.4rem;
}

.weather-card {
  background: var(--color-bg, #111);
  border: 1px solid var(--color-border, #333);
  border-radius: 4px;
  padding: 0.4rem 0.6rem;
  min-width: 72px;
  font-size: 0.78rem;
  text-align: center;
}

.wc-time { color: var(--color-muted, #888); font-size: 0.72rem; }
.wc-temp { font-weight: 600; }
.wc-precip { color: #60a5fa; }
.wc-wind { color: #86efac; }

.link-row {
  display: flex;
  gap: 0.4rem;
  align-items: center;
}
.link-label { width: 140px; }
.link-url { flex: 1; }
.link-cat { width: 80px; }

.checklist-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}
.chk-label { flex: 1; }

.checklist-add {
  display: flex;
  gap: 0.5rem;
  margin-top: 0.4rem;
}
.checklist-add input { flex: 1; }

textarea {
  width: 100%;
  box-sizing: border-box;
  resize: vertical;
}

.ai-section { border-color: #4338ca; }

.ai-summary-box {
  background: var(--color-bg, #111);
  border-radius: 4px;
  padding: 0.6rem;
}

.ai-disclaimer {
  font-size: 0.78rem;
  color: var(--color-muted, #888);
  border-top: 1px solid var(--color-border, #333);
  padding-top: 0.5rem;
  margin-top: 0.4rem;
}

.hint {
  font-size: 0.8rem;
  color: var(--color-muted, #888);
  margin: 0;
}

.rp-actions {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  align-items: flex-start;
}

.btn-sm { font-size: 0.8rem; padding: 0.2rem 0.5rem; }
.btn-danger { color: #f87171; border-color: #f87171; }
</style>
