<template>
  <div class="repeater-panel">
    <h4>附近中继台
      <button class="btn btn-sm" :disabled="loading || !canSearch" @click="search">
        {{ loading ? '搜索中…' : '搜索' }}
      </button>
    </h4>
    <p v-if="!canSearch" class="hint">请先在上方填写经纬度。</p>
    <p v-if="error" class="banner banner-error">{{ error }}</p>

    <div v-if="repeaters.length" class="repeater-list">
      <div v-for="r in repeaters" :key="r.StateID" class="repeater-row">
        <div class="rep-freq">{{ r.Frequency }} MHz</div>
        <div class="rep-info">
          <span class="rep-call">{{ r.Callsign }}</span>
          <span class="hint">{{ r.Nearest_City }}{{ r.State ? `, ${r.State}` : '' }}</span>
        </div>
        <div class="rep-tone">
          <span v-if="r.PL" class="tone-badge">{{ r.PL }} Hz</span>
          <span v-if="r.Digital_Code" class="tone-badge dmr">CC{{ r.Digital_Code }}</span>
        </div>
        <div class="rep-use">{{ r.Use }}</div>
      </div>
    </div>
    <p v-else-if="searched && !loading" class="hint">该位置附近未找到中继台。</p>

    <p class="hint source-hint">
      数据来源：
      <button class="link-btn" @click="openRepeaterBook">RepeaterBook</button>
      · 仅供参考，请以当地无线电协会数据为准。
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { open as shellOpen } from '@tauri-apps/plugin-shell'

const props = defineProps<{
  latitude: number | null
  longitude: number | null
}>()

interface Repeater {
  StateID: string
  Callsign: string
  Frequency: string
  PL: string
  Digital_Code: string
  Nearest_City: string
  State: string
  Use: string
  Operational_Status: string
}

const repeaters = ref<Repeater[]>([])
const loading = ref(false)
const error = ref('')
const searched = ref(false)

const canSearch = computed(() => props.latitude != null && props.longitude != null)

async function search() {
  if (!canSearch.value) return
  loading.value = true
  error.value = ''
  searched.value = false
  try {
    const url = new URL('https://www.repeaterbook.com/api/export.php')
    url.searchParams.set('lat', String(props.latitude))
    url.searchParams.set('lng', String(props.longitude))
    url.searchParams.set('distance', '50')
    url.searchParams.set('Dunit', 'km')
    url.searchParams.set('format', 'json')

    const res = await fetch(url.toString(), { headers: { Accept: 'application/json' } })
    if (!res.ok) throw new Error(`RepeaterBook API ${res.status}`)
    const json = await res.json()
    repeaters.value = (json.results ?? []).slice(0, 20) as Repeater[]
    searched.value = true
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    // Fall back to browser link
  } finally {
    loading.value = false
  }
}

async function openRepeaterBook() {
  const url = props.latitude != null && props.longitude != null
    ? `https://www.repeaterbook.com/repeaters/nearest.php?lat=${props.latitude}&long=${props.longitude}&distance=50&Dunit=km&band1=%25&freq=&call=&use=OPEN&status_id=1&order=distance_calc%2C+%60callsign%60+ASC&submit=Search`
    : 'https://www.repeaterbook.com'
  try { await shellOpen(url) } catch { window.open(url, '_blank') }
}
</script>

<style scoped>
.repeater-panel {
  margin-top: .75rem;
  border-top: 1px solid var(--color-border, #333);
  padding-top: .75rem;
}

.repeater-panel h4 {
  margin: 0 0 .5rem;
  font-size: .88rem;
  display: flex;
  align-items: center;
  gap: .5rem;
}

.repeater-list {
  display: flex;
  flex-direction: column;
  gap: .3rem;
  margin-top: .5rem;
}

.repeater-row {
  display: flex;
  gap: .6rem;
  align-items: center;
  font-size: .82rem;
  padding: .25rem .4rem;
  background: var(--color-bg, #111);
  border-radius: 4px;
}

.rep-freq { font-weight: 600; min-width: 88px; font-variant-numeric: tabular-nums; }
.rep-info { flex: 1; display: flex; flex-direction: column; gap: .1rem; }
.rep-call { font-weight: 600; }

.rep-tone { display: flex; gap: .3rem; }
.tone-badge {
  background: #1e3a5f;
  border-radius: 3px;
  padding: .1rem .35rem;
  font-size: .75rem;
}
.tone-badge.dmr { background: #3b1e5f; }

.rep-use {
  font-size: .75rem;
  color: var(--color-muted, #888);
  min-width: 40px;
  text-align: right;
}

.source-hint { margin-top: .5rem; }
.link-btn {
  background: none;
  border: none;
  color: #60a5fa;
  cursor: pointer;
  padding: 0;
  text-decoration: underline;
  font-size: inherit;
}
.hint { font-size: .78rem; color: var(--color-muted, #888); margin: .2rem 0; }
</style>
