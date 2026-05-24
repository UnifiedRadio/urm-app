// URC-v1 TypeScript types — mirrors crates/urm-core/src/schema/
// Keep in sync with urc-schema/schema/urc-v1.json

export type ChannelMode = 'analog_fm' | 'dmr' | 'c4fm' | 'dstar'
export type PowerLevel = 'high' | 'medium' | 'low'
export type Bandwidth = 'wide' | 'narrow'
export type ToneType = 'none' | 'ctcss' | 'dcs'
export type OwnerType = 'personal' | 'team' | 'activity'

export interface AnalogFields {
  tone_type: ToneType
  tone_value_hz?: number
  scan: boolean
}

export interface DmrFields {
  color_code: number       // 0-15
  time_slot: 1 | 2
  talkgroup_id: number
  contact_ref: string
  rx_group_ref: string
  zone_ref: string
}

export interface Channel {
  id: string
  name: string             // max 16 chars for most radios
  mode: ChannelMode
  rx_freq_mhz: number
  tx_freq_mhz: number
  power: PowerLevel
  bandwidth: Bandwidth
  tags: string[]
  analog?: AnalogFields
  dmr?: DmrFields
}

export interface DeviceBinding {
  model: string            // e.g. "baofeng_uv5r"
  transport: 'serial' | 'ble' | 'otg'
  adapter: 'chirp' | 'xiaomi_ble' | 'qdmr' | 'editcp' | 'dmrconfig'
}

export interface UrcProfile {
  urc_version: string      // "1.0"
  id: string
  name: string
  region?: string          // ISO 3166 country code
  owner_type: OwnerType
  compliance_mode: boolean
  channels: Channel[]
  devices: DeviceBinding[]
}

// Tauri IPC response types

export interface PortInfo {
  name: string
  description?: string
  vid?: number
  pid?: number
}

export interface PortProbeResult {
  port: string
  accessible: boolean
  driver: 'genuine' | 'counterfeit_prolific' | 'counterfeit_ftdi' | 'ch340' | 'unknown'
  error?: string
}

export interface BleDevice {
  id: string
  name?: string
  rssi?: number
}

export interface WriteResult {
  success: boolean
  channels_written: number
  warnings: string[]
}

export interface ValidationItem {
  channel_id: string | null
  field: string
  message: string
}

export interface ValidationResult {
  valid: boolean
  errors: ValidationItem[]
  warnings: ValidationItem[]
}

export type BackupKind = 'device_image' | 'profile_snapshot'

export interface BackupMeta {
  id: string
  profile_id: string
  path: string
  device_model: string
  kind: BackupKind
  created_at: string   // ISO 8601
  size_bytes: number
}

export interface DeviceCatalogEntry {
  urm_id: string
  chirp_driver: string | null
  vendor: string
  model: string
  aliases: string[]
  freq_ranges_mhz: [number, number][]
  max_channels: number
  max_name_chars: number
  valid_modes: string[]
  transport: 'serial' | 'ble' | 'otg'
  baud_rate: number | null
  adapter: DeviceBinding['adapter'] | null
}

export interface DeviceCatalog {
  schema_version: string
  generated_at: string
  source: string
  total_models: number
  models: DeviceCatalogEntry[]
}

export interface DeviceUiOverlay {
  display_name: string
  description_zh: string
  category: 'beginner' | 'intermediate' | 'advanced'
  featured?: boolean
  sort_order?: number
}
