mod commands;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use urm_adapter_chirp::ChirpAdapter;
use urm_core::backup::BackupManager;
use urm_core::catalog::device_catalog;
use urm_core::router::AdapterRouter;
use urm_core::task_queue::TaskQueue;

pub struct AppState {
    pub router: Arc<Mutex<AdapterRouter>>,
    pub task_queue: Arc<TaskQueue>,
}

impl AppState {
    fn init() -> Self {
        // Backup directory: platform data dir / openradio / backups
        let backup_dir = dirs_next::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("openradio")
            .join("backups");

        let backup_manager = Arc::new(BackupManager::new(backup_dir));
        let task_queue = Arc::new(TaskQueue::new(backup_manager));

        let mut router = AdapterRouter::new();
        register_catalog_chirp_adapters(&mut router);

        AppState {
            router: Arc::new(Mutex::new(router)),
            task_queue,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    urm_core::log::init("info");

    let state = AppState::init();

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::device::list_serial_ports,
            commands::device::diagnose_serial,
            commands::device::read_device,
            commands::device::write_device,
            commands::device::backup_device,
            commands::ble::scan_ble_devices,
            commands::ble::connect_ble,
            commands::ble::read_ble_device,
            commands::ble::write_ble_device,
            commands::config::import_chirp_csv,
            commands::config::export_chirp_csv,
            commands::config::import_urc_json,
            commands::config::export_urc_json,
            commands::config::import_urc_yaml,
            commands::config::export_urc_yaml,
            commands::config::validate_profile,
            commands::config::validate_for_device,
            commands::config::save_profile_snapshot,
            commands::config::list_backups,
            commands::config::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn register_catalog_chirp_adapters(router: &mut AdapterRouter) {
    for entry in device_catalog().serial_chirp_models() {
        let adapter = ChirpAdapter::new(
            entry.urm_id.clone(),
            entry.vendor.clone(),
            entry.model.clone(),
        );
        router.register(Arc::new(adapter));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_chirp_adapters_from_catalog() {
        let mut router = AdapterRouter::new();
        register_catalog_chirp_adapters(&mut router);
        assert!(router.resolve(&"baofeng_uv5r".to_string()).is_some());
    }
}
