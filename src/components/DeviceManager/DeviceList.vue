<template>
  <div class="device-list">
    <section>
      <h2>串口设备</h2>
      <button @click="deviceStore.refreshSerialPorts()">刷新</button>
      <ul>
        <li v-for="p in deviceStore.serialPorts" :key="p.name">
          {{ p.name }} {{ p.description ? `— ${p.description}` : '' }}
          <button @click="deviceStore.probePort(p.name)">诊断</button>
        </li>
        <li v-if="!deviceStore.serialPorts.length">未检测到串口设备</li>
      </ul>
    </section>
    <section>
      <h2>蓝牙设备</h2>
      <button @click="deviceStore.refreshBleDevices()" :disabled="deviceStore.isScanning">
        {{ deviceStore.isScanning ? '扫描中…' : '扫描蓝牙' }}
      </button>
      <ul>
        <li v-for="d in deviceStore.bleDevices" :key="d.id">
          {{ d.name ?? d.id }}
        </li>
        <li v-if="!deviceStore.bleDevices.length">未找到蓝牙设备</li>
      </ul>
    </section>
  </div>
</template>
<script setup lang="ts">
import { useDeviceStore } from '../../store/devices'
const deviceStore = useDeviceStore()
</script>
