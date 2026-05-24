import { invoke } from '@tauri-apps/api/core'
import type { BleDevice, UrcProfile } from '../types/urc'

export const scanBleDevices = () =>
  invoke<BleDevice[]>('scan_ble_devices')

export const connectBle = (deviceId: string) =>
  invoke<void>('connect_ble', { device_id: deviceId })

export const readBleDevice = (deviceId: string) =>
  invoke<UrcProfile>('read_ble_device', { device_id: deviceId })

export const writeBleDevice = (deviceId: string, profile: UrcProfile) =>
  invoke<void>('write_ble_device', { device_id: deviceId, profile })
