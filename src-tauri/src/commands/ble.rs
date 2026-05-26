use serde::Serialize;
use urm_adapter_xiaomi::{scan_xiaomi_devices, urc_to_write_params, BleSession};
use urm_core::schema::UrcProfile;

#[derive(Serialize)]
pub struct BleDevice {
    pub id: String,
    pub name: Option<String>,
    pub rssi: Option<i16>,
}

#[tauri::command]
pub async fn scan_ble_devices() -> Result<Vec<BleDevice>, String> {
    let devices = scan_xiaomi_devices().await.map_err(|e| e.to_string())?;
    Ok(devices
        .into_iter()
        .map(|d| BleDevice { id: d.id, name: d.name, rssi: d.rssi })
        .collect())
}

#[tauri::command]
pub async fn connect_ble(device_id: String) -> Result<(), String> {
    let session = BleSession::connect(&device_id).await.map_err(|e| e.to_string())?;
    session.disconnect().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_ble_device(device_id: String) -> Result<UrcProfile, String> {
    let mut session = BleSession::connect(&device_id).await.map_err(|e| e.to_string())?;
    let profile = session.read_profile(&device_id).await.map_err(|e| e.to_string())?;
    session.disconnect().await.ok();
    Ok(profile)
}

#[tauri::command]
pub async fn write_ble_device(device_id: String, profile: UrcProfile) -> Result<(), String> {
    let mut session = BleSession::connect(&device_id).await.map_err(|e| e.to_string())?;
    for (seq, rx_hz, tx_hz, rx_css, tx_css) in urc_to_write_params(&profile) {
        session
            .write_channel(seq, rx_hz, tx_hz, rx_css, tx_css)
            .await
            .map_err(|e| format!("channel {seq}: {e}"))?;
    }
    session.disconnect().await.ok();
    Ok(())
}
