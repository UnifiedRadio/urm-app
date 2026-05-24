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

export interface ActivityRow {
  id: string
  org_id: string | null
  creator_id: string
  name: string
  description: string | null
  status: 'planning' | 'active' | 'archived'
  device_model: string | null
  start_date: string | null
  end_date: string | null
  created_at: string
}
