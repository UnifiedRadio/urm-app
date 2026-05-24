use tauri::State;
use urm_core::backup::BackupMeta;
use urm_core::schema::UrcProfile;
use urm_core::schema::validation::{validate_profile as core_validate, validate_profile_for_device as core_validate_for_device};
use serde::Serialize;
use crate::AppState;

#[derive(Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// ── Import / Export ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn import_chirp_csv(path: String) -> Result<UrcProfile, String> {
    let raw = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
    // Try text CSV first (covers CHIRP File→Export→CSV), fall back to binary .img
    if let Ok(text) = std::str::from_utf8(&raw) {
        if let Ok(profile) = urm_adapter_chirp::converter::chirp_csv_to_urc(text) {
            return Ok(profile);
        }
    }
    urm_adapter_chirp::converter::chirp_img_to_urc(&raw).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_chirp_csv(profile: UrcProfile, path: String) -> Result<(), String> {
    let csv = urm_adapter_chirp::converter::urc_to_chirp_csv(&profile)
        .map_err(|e| e.to_string())?;
    tokio::fs::write(&path, csv.as_bytes()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_urc_json(path: String) -> Result<UrcProfile, String> {
    let text = tokio::fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_urc_json(profile: UrcProfile, path: String) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, json.as_bytes()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_urc_yaml(path: String) -> Result<UrcProfile, String> {
    let text = tokio::fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
    serde_yaml::from_str(&text).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_urc_yaml(profile: UrcProfile, path: String) -> Result<(), String> {
    let yaml = serde_yaml::to_string(&profile).map_err(|e| e.to_string())?;
    tokio::fs::write(&path, yaml.as_bytes()).await.map_err(|e| e.to_string())
}

// ── Validation ───────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn validate_profile(profile: UrcProfile) -> Result<ValidationResult, String> {
    let result = core_validate(&profile);
    Ok(ValidationResult {
        valid: result.is_valid(),
        errors: result.errors.iter().map(|e| e.message.clone()).collect(),
        warnings: result.warnings.iter().map(|w| w.message.clone()).collect(),
    })
}

#[tauri::command]
pub async fn validate_for_device(profile: UrcProfile, model: String) -> Result<ValidationResult, String> {
    let result = core_validate_for_device(&profile, &model);
    Ok(ValidationResult {
        valid: result.is_valid(),
        errors: result.errors.iter().map(|e| e.message.clone()).collect(),
        warnings: result.warnings.iter().map(|w| w.message.clone()).collect(),
    })
}

// ── Backup / Version history (FR-04) ─────────────────────────────────────────

#[tauri::command]
pub async fn save_profile_snapshot(
    profile: UrcProfile,
    state: State<'_, AppState>,
) -> Result<BackupMeta, String> {
    state.task_queue
        .backup_manager()
        .save_snapshot(&profile)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupMeta>, String> {
    state.task_queue
        .backup_manager()
        .list()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_backup(
    backup_id: String,
    state: State<'_, AppState>,
) -> Result<UrcProfile, String> {
    state.task_queue
        .backup_manager()
        .restore_snapshot(&backup_id)
        .await
        .map_err(|e| e.to_string())
}
