import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { PortInfo, PortProbeResult, BleDevice } from '../types/urc'
import { listSerialPorts, diagnoseSerial } from '../api/device'
import { scanBleDevices } from '../api/ble'

export const useDeviceStore = defineStore('devices', () => {
  const serialPorts = ref<PortInfo[]>([])
  const bleDevices = ref<BleDevice[]>([])
  const probeResult = ref<PortProbeResult | null>(null)
  const isScanning = ref(false)

  async function refreshSerialPorts() {
    serialPorts.value = await listSerialPorts()
    return serialPorts.value
  }

  async function probePort(port: string) {
    probeResult.value = await diagnoseSerial(port)
    return probeResult.value
  }

  async function refreshBleDevices() {
    isScanning.value = true
    try {
      bleDevices.value = await scanBleDevices()
    } finally {
      isScanning.value = false
    }
  }

  return { serialPorts, bleDevices, probeResult, isScanning, refreshSerialPorts, probePort, refreshBleDevices }
})
