<template>
  <div class="dmr-form">
    <label>色码 (Color Code)
      <select v-model.number="local.color_code">
        <option v-for="n in 16" :key="n - 1" :value="n - 1">{{ n - 1 }}</option>
      </select>
    </label>
    <label>时隙 (Time Slot)
      <select v-model.number="local.time_slot">
        <option :value="1">Slot 1</option>
        <option :value="2">Slot 2</option>
      </select>
    </label>
    <label>通话组 ID (Talk Group ID)
      <input type="number" v-model.number="local.talkgroup_id" min="1" max="16776415" />
    </label>
    <label>联系人引用 (Contact Ref)
      <input v-model="local.contact_ref" placeholder="Contact name or ID" />
    </label>
    <label>接收组引用 (RX Group Ref)
      <input v-model="local.rx_group_ref" placeholder="RX group name or ID" />
    </label>
    <label>区域引用 (Zone Ref)
      <input v-model="local.zone_ref" placeholder="Zone name or ID" />
    </label>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'
import type { DmrFields } from '../../types/urc'

const props = defineProps<{ modelValue: DmrFields }>()
const emit = defineEmits<{ 'update:modelValue': [v: DmrFields] }>()

const local = reactive<DmrFields>({ ...props.modelValue })
watch(local, v => emit('update:modelValue', { ...v }))
</script>
