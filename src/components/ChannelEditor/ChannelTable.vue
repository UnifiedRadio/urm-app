<template>
  <div class="channel-table-wrap">
    <table class="channel-table">
      <thead>
        <tr>
          <th>#</th>
          <th>名称</th>
          <th>模式</th>
          <th>接收 MHz</th>
          <th>发射 MHz</th>
          <th>亚音</th>
          <th>功率</th>
          <th>带宽</th>
          <th>标签</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(ch, i) in profile.channels"
          :key="ch.id"
          :class="{ 'row-invalid': isOutOfRange(ch) }"
        >
          <td class="col-num dim">{{ i + 1 }}</td>
          <td class="col-name">
            {{ ch.name }}
            <span v-if="ch.name.length > maxNameChars" class="tag-warn" :title="`超出 ${maxNameChars} 字符硬件限制`">!</span>
          </td>
          <td>{{ modeLabel(ch.mode) }}</td>
          <td class="col-freq" :class="{ 'freq-warn': isOutOfRange(ch) }">
            {{ ch.rx_freq_mhz.toFixed(4) }}
            <span v-if="isOutOfRange(ch)" class="tag-warn" title="频率超出设备支持范围">⚠</span>
          </td>
          <td class="col-freq">{{ ch.tx_freq_mhz.toFixed(4) }}</td>
          <td class="col-tone">{{ toneLabel(ch) }}</td>
          <td>{{ ch.power[0].toUpperCase() }}</td>
          <td>{{ ch.bandwidth === 'narrow' ? 'N' : 'W' }}</td>
          <td class="col-tags">
            <span v-for="tag in ch.tags" :key="tag" class="tag">{{ tag }}</span>
          </td>
          <td class="col-actions">
            <button class="btn-sm" @click="emit('edit', ch)">编辑</button>
            <button class="btn-sm btn-danger" @click="emit('delete', ch)">删除</button>
          </td>
        </tr>
        <tr v-if="!profile.channels.length">
          <td colspan="10" class="empty-row">
            暂无频道 — 点击"+ 添加频道"开始配置。
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { UrcProfile, Channel, ChannelMode } from '../../types/urc'
import { isFreqValid, DEVICE_TEMPLATES } from '../../data/deviceTemplates'

const props = defineProps<{
  profile: UrcProfile
  boundModel?: string
}>()
const emit = defineEmits<{
  edit: [channel: Channel]
  delete: [channel: Channel]
}>()

const maxNameChars = computed(() => {
  if (!props.boundModel) return 16
  const tpl = DEVICE_TEMPLATES.find(t => t.model === props.boundModel)
  return tpl?.max_name_chars ?? 16
})

function isOutOfRange(ch: Channel): boolean {
  if (!props.boundModel || ch.rx_freq_mhz <= 0) return false
  return !isFreqValid(ch.rx_freq_mhz, props.boundModel)
}

function toneLabel(ch: Channel): string {
  const a = ch.analog
  if (!a || a.tone_type === 'none') return '—'
  if (a.tone_type === 'ctcss') return `${a.tone_value_hz ?? 88.5} Hz`
  return 'DCS'
}

const MODE_LABELS: Record<ChannelMode, string> = {
  analog_fm: 'FM',
  dmr: 'DMR',
  c4fm: 'C4FM',
  dstar: 'D-STAR',
}
function modeLabel(mode: ChannelMode) { return MODE_LABELS[mode] ?? mode }
</script>
