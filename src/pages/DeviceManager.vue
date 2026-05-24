<template>
  <div class="page page-device-manager">
    <div class="device-layout">
      <DeviceList />
      <DiagnosisWizard />
    </div>

    <section class="write-panel">
      <h2>UV-5R Safe Write</h2>
      <p v-if="!channelStore.profile" class="banner banner-warn">Load a profile before running write checks.</p>
      <div class="form-row">
        <label>Port
          <select v-model="selectedPort">
            <option value="">Select port</option>
            <option v-for="p in deviceStore.serialPorts" :key="p.name" :value="p.name">
              {{ p.name }}
            </option>
          </select>
        </label>
        <label>Model
          <input v-model="model" />
        </label>
      </div>
      <div class="form-row">
        <button class="btn" :disabled="!channelStore.profile || running" @click="runMockWrite">
          Mock dry-run
        </button>
        <button class="btn-warn" :disabled="!canWrite || running" @click="runSafeWrite">
          Backup and write
        </button>
      </div>
      <WriteProgress :steps="steps" />
      <p v-if="message" class="banner" :class="messageType === 'error' ? 'banner-error' : 'banner-warn'">{{ message }}</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import DeviceList from '../components/DeviceManager/DeviceList.vue'
import DiagnosisWizard from '../components/DeviceManager/DiagnosisWizard.vue'
import WriteProgress from '../components/DeviceManager/WriteProgress.vue'
import { useDeviceStore } from '../store/devices'
import { useChannelStore } from '../store/channels'
import { backupDevice, writeDevice } from '../api/device'
import { validateForDevice } from '../api/config'

type StepStatus = 'pending' | 'running' | 'ok' | 'error'

const deviceStore = useDeviceStore()
const channelStore = useChannelStore()
const selectedPort = ref('')
const model = ref('baofeng_uv5r')
const running = ref(false)
const message = ref('')
const messageType = ref<'warning' | 'error'>('warning')

const steps = ref([
  { label: '串口枚举', status: 'pending' as StepStatus },
  { label: '驱动识别', status: 'pending' as StepStatus },
  { label: 'Dry-run 校验', status: 'pending' as StepStatus },
  { label: '备份', status: 'pending' as StepStatus },
  { label: '写入', status: 'pending' as StepStatus },
])

const canWrite = computed(() => Boolean(channelStore.profile && selectedPort.value && model.value))

function resetSteps() {
  for (const step of steps.value) step.status = 'pending'
}

function mark(label: string, status: StepStatus) {
  const step = steps.value.find(s => s.label === label)
  if (step) step.status = status
}

async function runMockWrite() {
  if (!channelStore.profile) return
  running.value = true
  message.value = ''
  resetSteps()
  try {
    mark('串口枚举', 'ok')
    mark('驱动识别', 'ok')
    mark('Dry-run 校验', 'running')
    const validation = await validateForDevice(channelStore.profile, model.value)
    if (!validation.valid) {
      mark('Dry-run 校验', 'error')
      throw new Error(validation.errors.map(e => e.message).join('; '))
    }
    mark('Dry-run 校验', 'ok')
    mark('备份', 'ok')
    mark('写入', 'ok')
    messageType.value = 'warning'
    message.value = 'Mock dry-run complete. No hardware write was attempted.'
  } catch (e) {
    messageType.value = 'error'
    message.value = e instanceof Error ? e.message : String(e)
  } finally {
    running.value = false
  }
}

async function runSafeWrite() {
  if (!channelStore.profile || !canWrite.value) return
  if (!confirm('This will read a backup before writing to the radio. Continue?')) return
  running.value = true
  message.value = ''
  resetSteps()
  try {
    mark('串口枚举', 'running')
    const ports = await deviceStore.refreshSerialPorts()
    const hasSelectedPort = ports.some(p => p.name === selectedPort.value)
    if (!hasSelectedPort) throw new Error('Selected serial port is no longer available.')
    mark('串口枚举', 'ok')
    mark('驱动识别', 'running')
    const probe = await deviceStore.probePort(selectedPort.value)
    if (!probe.accessible) throw new Error(probe.error ?? 'Selected port is not accessible.')
    mark('驱动识别', 'ok')
    mark('Dry-run 校验', 'running')
    const validation = await validateForDevice(channelStore.profile, model.value)
    if (!validation.valid) throw new Error(validation.errors.map(e => e.message).join('; '))
    mark('Dry-run 校验', 'ok')
    mark('备份', 'running')
    await backupDevice(selectedPort.value, model.value)
    mark('备份', 'ok')
    mark('写入', 'running')
    await writeDevice(selectedPort.value, model.value, channelStore.profile)
    mark('写入', 'ok')
    messageType.value = 'warning'
    message.value = 'Write command completed. Run a read-back check before field use.'
  } catch (e) {
    const runningStep = steps.value.find(s => s.status === 'running')
    if (runningStep) runningStep.status = 'error'
    messageType.value = 'error'
    message.value = e instanceof Error ? e.message : String(e)
  } finally {
    running.value = false
  }
}
</script>
