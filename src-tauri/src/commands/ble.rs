use urm_core::schema::UrcProfile;
use serde::Serialize;

#[derive(Serialize)]
pub struct BleDevice {
    pub id: String,
    pub name: Option<String>,
    pub rssi: Option<i16>,
}

#[tauri::command]
pub async fn scan_ble_devices() -> Result<Vec<BleDevice>, String> {
    // TODO Phase 3: use btleplug to scan and filter Xiaomi WT2 devices
    Err("BLE scan not yet implemented — Phase 3".into())
}

#[tauri::command]
pub async fn connect_ble(device_id: String) -> Result<(), String> {
    let _ = device_id;
    Err("BLE connect not yet implemented — Phase 3".into())
}

#[tauri::command]
pub async fn read_ble_device(device_id: String) -> Result<UrcProfile, String> {
    let _ = device_id;
    Err("BLE read not yet implemented — Phase 3".into())
}

#[tauri::command]
pub async fn write_ble_device(device_id: String, profile: UrcProfile) -> Result<(), String> {
    let _ = (device_id, profile);
    Err("BLE write not yet implemented — Phase 3".into())
}
