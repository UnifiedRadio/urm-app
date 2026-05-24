import catalogJson from '../../../urc-schema/data/devices.json'
import type {
  DeviceCatalog,
  DeviceCatalogEntry,
  DeviceBinding,
  DeviceUiOverlay,
  UrcProfile,
} from '../types/urc'

export interface FreqRange {
  min_mhz: number
  max_mhz: number
  label: string
}

export interface DeviceTemplate {
  id: string
  name: string
  model: string
  adapter: DeviceBinding['adapter']
  transport: 'serial' | 'ble' | 'otg'
  category: 'beginner' | 'intermediate' | 'advanced'
  description: string
  freq_ranges: FreqRange[]
  max_channels: number
  max_name_chars: number
  supported_modes: string[]
  starter_profile: UrcProfile
}

const DEVICE_CATALOG = catalogJson as unknown as DeviceCatalog

const OVERLAY: Record<string, DeviceUiOverlay> = {
  baofeng_bf888s: {
    display_name: 'Baofeng BF-888S',
    description_zh: '常见入门 UHF 机型，16 信道，串口写频。',
    category: 'beginner',
    featured: true,
    sort_order: 10,
  },
  baofeng_uv5r: {
    display_name: 'Baofeng UV-5R / UV-5RA / UV-5RB',
    description_zh: '流行双频入门机型，VHF + UHF，128 信道。',
    category: 'beginner',
    featured: true,
    sort_order: 20,
  },
  xiaomi_wt2: {
    display_name: '小米对讲机 2 (Xiaomi WT2)',
    description_zh: '蓝牙写频机型，无需写频线。',
    category: 'beginner',
    featured: true,
    sort_order: 30,
  },
  retevis_h777: {
    display_name: 'Retevis H777 / H-777',
    description_zh: '与 BF-888S 同类的 16 信道 UHF 机型。',
    category: 'beginner',
    featured: true,
    sort_order: 40,
  },
  baofeng_uv82: {
    display_name: 'Baofeng UV-82 / UV-82HP',
    description_zh: '双频手台，128 信道。',
    category: 'intermediate',
    featured: true,
    sort_order: 50,
  },
  baofeng_uv5x3: {
    display_name: 'BTECH UV-5X3',
    description_zh: '三频机型（含 1.25m 频段）。',
    category: 'intermediate',
    featured: true,
    sort_order: 60,
  },
  yaesu_ft65: {
    display_name: 'Yaesu FT-65 / FT-4XR',
    description_zh: '八重洲入门双频手台，200 信道。',
    category: 'intermediate',
    featured: true,
    sort_order: 70,
  },
  retevis_rt85: {
    display_name: 'Retevis RT85',
    description_zh: '双频 200 信道手台，常见于队伍协作场景。',
    category: 'intermediate',
    featured: true,
    sort_order: 80,
  },
  baofeng_uvu10: {
    display_name: '宝锋 UV-U10 猎鹰',
    description_zh: '双频手台（VHF+UHF），128 信道，10W，IP45，4000mAh+，USB 充电。写频需另购 K 头线，CHIRP 驱动待确认。',
    category: 'beginner',
    featured: true,
    sort_order: 22,
  },
  jifeng_a208: {
    display_name: '极蜂 A208 小语',
    description_zh: '迷你超小型 UHF 对讲机，16 信道。同品牌 A108Plus 为蓝牙写频，A208 写频方式待确认，当前仅供配置管理使用。',
    category: 'beginner',
    featured: true,
    sort_order: 45,
  },
  jifeng_a108plus: {
    display_name: '极蜂小语 A108Plus',
    description_zh: '蓝牙 App 写频，无需写频线。UHF 430–440 MHz，16 信道，5W，3350mAh。BLE 协议待逆向，当前仅供配置管理与频率校验使用。',
    category: 'beginner',
    featured: true,
    sort_order: 47,
  },
}

function bandLabel(min: number, max: number, index: number): string {
  if (min >= 136 && max <= 174) return 'VHF'
  if (min >= 400 && max <= 520) return 'UHF'
  if (min >= 222 && max <= 225) return '1.25m'
  if (min >= 50 && max <= 54) return '6m'
  return `Band ${index + 1}`
}

function adapterFor(entry: DeviceCatalogEntry): DeviceBinding['adapter'] {
  if (entry.transport === 'ble') return 'xiaomi_ble'
  if (entry.chirp_driver) return 'chirp'
  return 'chirp'
}

function mapMode(mode: string): string {
  const upper = mode.toUpperCase()
  if (upper === 'DMR') return 'dmr'
  return 'analog_fm'
}

function makeProfile(
  name: string,
  model: string,
  adapter: DeviceBinding['adapter'],
  transport: 'serial' | 'ble' | 'otg',
): UrcProfile {
  return {
    urc_version: '1.0',
    id: crypto.randomUUID(),
    name,
    owner_type: 'personal',
    compliance_mode: false,
    channels: [],
    devices: [{ model, transport, adapter }],
  }
}

function toTemplate(entry: DeviceCatalogEntry): DeviceTemplate {
  const overlay = OVERLAY[entry.urm_id]
  const name = overlay?.display_name ?? `${entry.vendor} ${entry.model}`
  const adapter = adapterFor(entry)
  const freqRanges = entry.freq_ranges_mhz.map(([min, max], idx) => ({
    min_mhz: min,
    max_mhz: max,
    label: bandLabel(min, max, idx),
  }))

  return {
    id: entry.urm_id,
    name,
    model: entry.urm_id,
    adapter,
    transport: entry.transport,
    category: overlay?.category ?? 'advanced',
    description: overlay?.description_zh ?? `${entry.vendor} ${entry.model} 机型模板。`,
    freq_ranges: freqRanges,
    max_channels: entry.max_channels,
    max_name_chars: entry.max_name_chars,
    supported_modes: Array.from(new Set(entry.valid_modes.map(mapMode))),
    starter_profile: makeProfile(entry.model, entry.urm_id, adapter, entry.transport),
  }
}

function compareTemplates(a: DeviceTemplate, b: DeviceTemplate): number {
  const aOrder = OVERLAY[a.model]?.sort_order ?? 10_000
  const bOrder = OVERLAY[b.model]?.sort_order ?? 10_000
  if (aOrder !== bOrder) return aOrder - bOrder
  return a.name.localeCompare(b.name)
}

export const DEVICE_TEMPLATES: DeviceTemplate[] = DEVICE_CATALOG.models
  .map(toTemplate)
  .sort(compareTemplates)

const TEMPLATE_LOOKUP = new Map<string, DeviceTemplate>()
for (const tpl of DEVICE_TEMPLATES) {
  TEMPLATE_LOOKUP.set(tpl.model, tpl)
  const raw = DEVICE_CATALOG.models.find(m => m.urm_id === tpl.model)
  for (const alias of raw?.aliases ?? []) TEMPLATE_LOOKUP.set(alias, tpl)
}

/** Look up frequency ranges for a given model ID (catalog-backed). */
export function freqRangesForModel(model: string): FreqRange[] {
  const tpl = TEMPLATE_LOOKUP.get(model)
  if (tpl) return tpl.freq_ranges
  return [{ min_mhz: 30, max_mhz: 1300, label: 'All' }]
}

/** Check if a frequency is within a model's supported ranges. */
export function isFreqValid(freqMhz: number, model: string): boolean {
  return freqRangesForModel(model).some(r => freqMhz >= r.min_mhz && freqMhz <= r.max_mhz)
}
