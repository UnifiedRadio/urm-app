<template>
  <div class="analog-form">
    <label>亚音类型
      <select v-model="local.tone_type">
        <option value="none">无</option>
        <option value="ctcss">CTCSS</option>
        <option value="dcs">DCS</option>
      </select>
    </label>
    <label v-if="local.tone_type !== 'none'">亚音频率 (Hz)
      <input type="number" v-model.number="local.tone_value_hz" step="0.1" />
    </label>
    <label>
      <input type="checkbox" v-model="local.scan" /> 参与扫描
    </label>
  </div>
</template>
<script setup lang="ts">
import { reactive, watch } from 'vue'
import type { AnalogFields } from '../../types/urc'
const props = defineProps<{ modelValue: AnalogFields }>()
const emit = defineEmits<{ 'update:modelValue': [v: AnalogFields] }>()
const local = reactive({ ...props.modelValue })
watch(local, v => emit('update:modelValue', { ...v }))
</script>
