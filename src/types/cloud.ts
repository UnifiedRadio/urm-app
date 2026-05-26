import type { UrcProfile } from './urc'

export interface CloudProfileRow {
  id: string
  owner_id: string
  org_id: string | null
  name: string
  urc_data: UrcProfile
  updated_at: string
  created_at: string
}

export interface SharePackResponse {
  url: string
  short_code: string
}

export interface SharePreview {
  short_code: string
  profile_id: string
  profile: UrcProfile
  expires_at: string | null
  view_count: number
}

export interface OrganizationRow {
  id: string
  name: string
  description: string | null
  owner_id: string
  invite_code: string | null
  created_at: string
}

export interface RiskPackWeatherHour {
  time: string
  temperature_2m: number
  precipitation_probability: number
  windspeed_10m: number
}

export interface RiskPackWeather {
  fetched_at: string
  source: 'open_meteo'
  forecast: RiskPackWeatherHour[]
}

export interface RiskPackSource {
  id: string
  label: string
  url: string
  category: 'weather' | 'disaster' | 'traffic' | 'local' | 'other'
  region?: string
}

export interface RiskPackChecklistItem {
  id: string
  text: string
  checked: boolean
}

export interface RiskPack {
  location_name: string
  latitude: number | null
  longitude: number | null
  activity_start: string | null
  activity_end: string | null
  weather_cache: RiskPackWeather | null
  official_links: RiskPackSource[]
  checklist: RiskPackChecklistItem[]
  captain_notes: string
  ai_summary: { text: string; generated_at: string } | null
}

export interface ActivityRow {
  id: string
  org_id: string | null
  creator_id: string
  name: string
  description: string | null
  status: 'planning' | 'active' | 'archived'
  location: { name?: string; lat?: number; lng?: number } | null
  device_model: string | null
  start_date: string | null
  end_date: string | null
  risk_pack: RiskPack | null
  created_at: string
}
