<template>
  <div class="diagnosis-wizard">
    <h2>连接诊断</h2>

    <!-- 串口选择 -->
    <div class="step step-port">
      <label>串口
        <select v-model="selectedPort">
          <option value="">— 选择串口 —</option>
          <option v-for="p in deviceStore.serialPorts" :key="p.name" :value="p.name">
            {{ p.name }}{{ p.description ? ` — ${p.description}` : '' }}
          </option>
        </select>
      </label>
      <button class="btn" @click="deviceStore.refreshSerialPorts()">刷新</button>
      <button
        class="btn-primary"
        :disabled="!selectedPort || diagnosing"
        @click="runDiagnosis"
      >
        {{ diagnosing ? '诊断中…' : '运行诊断' }}
      </button>
    </div>

    <!-- 诊断结果 -->
    <div v-if="result" class="diagnosis-result" :class="result.accessible ? 'ok' : 'error'">
      <div class="result-header">
        <span class="result-icon">{{ result.accessible ? '✓' : '✗' }}</span>
        <strong>{{ result.port }}</strong>
        <span class="driver-badge">{{ driverLabel(result.driver) }}</span>
      </div>

      <div v-if="result.error" class="result-error">{{ result.error }}</div>

      <div class="guidance">
        <template v-if="result.driver === 'counterfeit_prolific'">
          <p><strong>检测到仿冒 Prolific 芯片。</strong></p>
          <p>此写频线使用了假冒 PL2303，新版驱动会拒绝识别。解决方法：</p>
          <ul>
            <li>在 Windows 上使用旧版 Prolific 驱动（v3.3.2.102）</li>
            <li>换一根使用正品 CH340 芯片的写频线</li>
          </ul>
        </template>
        <template v-else-if="result.driver === 'counterfeit_ftdi'">
          <p><strong>检测到仿冒 FTDI 芯片。</strong></p>
          <p>FTDI 官方驱动可能损坏此芯片，建议换用 CH340 写频线。</p>
        </template>
        <template v-else-if="result.driver === 'ch340'">
          <p>检测到 CH340 写频线，是个好选择。请确认已安装驱动：Linux 上为 <code>ch34x</code>，macOS/Windows 安装对应 CH340 驱动即可。</p>
        </template>
        <template v-else-if="result.driver === 'genuine'">
          <p>检测到正品写频线。如果写频仍然失败，请检查：</p>
          <ul>
            <li>对讲机已开机且处于写频模式</li>
            <li>选择了正确的 COM/tty 串口</li>
            <li>CHIRP 已安装（<code>pip install chirp</code>）</li>
          </ul>
        </template>
        <template v-else-if="!result.accessible">
          <p>串口无法访问，可能原因：</p>
          <ul>
            <li>其他程序（CHIRP、串口调试工具）占用了该串口</li>
            <li>权限不足 — Linux 用户请执行 <code>sudo usermod -aG dialout $USER</code></li>
            <li>写频线未插好</li>
          </ul>
        </template>
        <template v-else>
          <p>串口已就绪。选择设备型号后点击"读取"即可下载频道配置。</p>
        </template>
      </div>
    </div>

    <div v-else-if="!diagnosing && deviceStore.serialPorts.length === 0" class="no-ports">
      未检测到串口设备。请插入写频线后点击"刷新"。
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useDeviceStore } from '../../store/devices'
import type { PortProbeResult } from '../../types/urc'

const deviceStore = useDeviceStore()
const selectedPort = ref('')
const diagnosing = ref(false)
const result = ref<PortProbeResult | null>(null)

const DRIVER_LABELS: Record<string, string> = {
  genuine: '正品',
  counterfeit_prolific: '仿冒 Prolific',
  counterfeit_ftdi: '仿冒 FTDI',
  ch340: 'CH340',
  unknown: '未知',
}

function driverLabel(driver: string): string {
  return DRIVER_LABELS[driver] ?? driver
}

async function runDiagnosis() {
  if (!selectedPort.value) return
  diagnosing.value = true
  result.value = null
  try {
    result.value = await deviceStore.probePort(selectedPort.value)
  } finally {
    diagnosing.value = false
  }
}
</script>
