import { invoke } from '@tauri-apps/api/core'
import type { UrcProfile, ValidationResult, BackupMeta } from '../types/urc'

// ── Import ───────────────────────────────────────────────────────────────────

export const importChirpCsv = (path: string) =>
  invoke<UrcProfile>('import_chirp_csv', { path })

export const importUrcJson = (path: string) =>
  invoke<UrcProfile>('import_urc_json', { path })

export const importUrcYaml = (path: string) =>
  invoke<UrcProfile>('import_urc_yaml', { path })

// ── Export ───────────────────────────────────────────────────────────────────

export const exportChirpCsv = (profile: UrcProfile, path: string) =>
  invoke<void>('export_chirp_csv', { profile, path })

export const exportUrcJson = (profile: UrcProfile, path: string) =>
  invoke<void>('export_urc_json', { profile, path })

export const exportUrcYaml = (profile: UrcProfile, path: string) =>
  invoke<void>('export_urc_yaml', { profile, path })

// ── Validation ───────────────────────────────────────────────────────────────

export const validateProfile = (profile: UrcProfile) =>
  invoke<ValidationResult>('validate_profile', { profile })

export const validateForDevice = (profile: UrcProfile, model: string) =>
  invoke<ValidationResult>('validate_for_device', { profile, model })

// ── Backup / Version history ──────────────────────────────────────────────────

export const saveProfileSnapshot = (profile: UrcProfile) =>
  invoke<BackupMeta>('save_profile_snapshot', { profile })

export const listBackups = () =>
  invoke<BackupMeta[]>('list_backups')

export const restoreBackup = (backupId: string) =>
  invoke<UrcProfile>('restore_backup', { backupId })
