import { invoke } from '@tauri-apps/api/core'
import type { PortInfo, PortProbeResult, UrcProfile, WriteResult, BackupMeta } from '../types/urc'

export const listSerialPorts = () =>
  invoke<PortInfo[]>('list_serial_ports')

export const diagnoseSerial = (port: string) =>
  invoke<PortProbeResult>('diagnose_serial', { port })

export const readDevice = (port: string, model: string) =>
  invoke<UrcProfile>('read_device', { port, model })

export const writeDevice = (port: string, model: string, profile: UrcProfile) =>
  invoke<WriteResult>('write_device', { port, model, profile })

export const backupDevice = (port: string, model: string) =>
  invoke<BackupMeta>('backup_device', { port, model })
