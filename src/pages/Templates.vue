<template>
  <div class="page page-templates">
    <div class="page-header">
      <h1>设备模板</h1>
      <p>选择你的对讲机型号，快速生成适配该机型的频道配置文件。</p>
    </div>

    <div v-for="cat in categories" :key="cat.id" class="template-category">
      <h2 class="category-title">{{ cat.label }}</h2>
      <div class="template-grid">
        <div
          v-for="tpl in cat.templates"
          :key="tpl.id"
          class="template-card"
        >
          <div class="template-card-header">
            <h3>{{ tpl.name }}</h3>
            <span class="transport-badge" :class="tpl.transport">
              {{ tpl.transport === 'ble' ? 'BLE' : '串口' }}
            </span>
          </div>

          <p class="template-desc">{{ tpl.description }}</p>

          <dl class="template-specs">
            <dt>频段</dt>
            <dd>
              <span v-for="r in tpl.freq_ranges" :key="r.label" class="freq-tag">
                {{ r.label }} {{ r.min_mhz }}–{{ r.max_mhz }}
              </span>
            </dd>
            <dt>最大信道</dt>
            <dd>{{ tpl.max_channels }} 个</dd>
            <dt>名称长度</dt>
            <dd>最多 {{ tpl.max_name_chars }} 字符</dd>
            <dt>适配器</dt>
            <dd>{{ tpl.adapter }}</dd>
          </dl>

          <button class="btn-primary tpl-apply" @click="applyTemplate(tpl)">
            使用此模板
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { DEVICE_TEMPLATES, type DeviceTemplate } from '../data/deviceTemplates'
import { useChannelStore } from '../store/channels'
import { useUiStore } from '../store/ui'

const channelStore = useChannelStore()
const ui = useUiStore()

const categories = [
  {
    id: 'beginner',
    label: '入门级',
    templates: DEVICE_TEMPLATES.filter(t => t.category === 'beginner'),
  },
  {
    id: 'intermediate',
    label: '进阶',
    templates: DEVICE_TEMPLATES.filter(t => t.category === 'intermediate'),
  },
  {
    id: 'advanced',
    label: '高级',
    templates: DEVICE_TEMPLATES.filter(t => t.category === 'advanced'),
  },
]

function applyTemplate(tpl: DeviceTemplate) {
  channelStore.setProfile({
    ...tpl.starter_profile,
    id: crypto.randomUUID(),
    name: `${tpl.name} – ${new Date().toLocaleDateString('zh-CN')}`,
    channels: [],
  })
  ui.navigate('channel-editor')
}
</script>
