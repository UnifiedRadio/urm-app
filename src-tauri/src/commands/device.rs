use tauri::State;
use urm_adapter_serial::{list_ports, probe_port, PortInfo, PortProbeResult};
use urm_core::schema::UrcProfile;
use urm_core::{WriteReport, BackupMeta};
use urm_core::catalog::device_catalog;
use crate::AppState;

#[tauri::command]
pub async fn list_serial_ports() -> Result<Vec<PortInfo>, String> {
    list_ports().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn diagnose_serial(port: String) -> Result<PortProbeResult, String> {
    let ports = list_ports().map_err(|e| e.to_string())?;
    let info = ports.iter().find(|p| p.name == port);
    let (vid, pid) = info.map(|p| (p.vid, p.pid)).unwrap_or((None, None));
    Ok(probe_port(&port, vid, pid))
}

#[tauri::command]
pub async fn read_device(
    port: String,
    model: String,
    state: State<'_, AppState>,
) -> Result<UrcProfile, String> {
    let (resolved_model, baud_rate) = resolve_model_and_baud(&model);
    let transport = urm_core::Transport::Serial { port, baud_rate };
    let adapter = state.router.resolve(&resolved_model)
        .ok_or_else(|| format!("no adapter registered for model '{resolved_model}'"))?;
    state.task_queue
        .read(adapter.as_ref(), &transport)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_device(
    port: String,
    model: String,
    profile: UrcProfile,
    state: State<'_, AppState>,
) -> Result<WriteReport, String> {
    let (resolved_model, baud_rate) = resolve_model_and_baud(&model);
    let transport = urm_core::Transport::Serial { port, baud_rate };
    let adapter = state.router.resolve(&resolved_model)
        .ok_or_else(|| format!("no adapter registered for model '{resolved_model}'"))?;
    state.task_queue
        .write(adapter.as_ref(), &transport, &profile, false)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backup_device(
    port: String,
    model: String,
    state: State<'_, AppState>,
) -> Result<BackupMeta, String> {
    let (resolved_model, baud_rate) = resolve_model_and_baud(&model);
    let transport = urm_core::Transport::Serial { port, baud_rate };
    let adapter = state.router.resolve(&resolved_model)
        .ok_or_else(|| format!("no adapter registered for model '{resolved_model}'"))?;
    state.task_queue
        .backup(adapter.as_ref(), &transport, "manual")
        .await
        .map_err(|e| e.to_string())
}

fn resolve_model_and_baud(model: &str) -> (String, u32) {
    if let Some(entry) = device_catalog().find_by_model_or_alias(model) {
        return (entry.urm_id.clone(), entry.baud_rate.unwrap_or(9600));
    }
    (model.to_string(), 9600)
}
