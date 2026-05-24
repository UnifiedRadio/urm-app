use async_trait::async_trait;
use urm_core::adapter::{
    AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport,
};
use urm_core::schema::{DeviceModel, UrcProfile};

static XIAOMI_LOCAL_NAME: &str = "Xiaomi WT2";

pub struct XiaomiAdapter;

#[async_trait]
impl RadioAdapter for XiaomiAdapter {
    fn name(&self) -> &str {
        "xiaomi_ble"
    }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec!["xiaomi_walkie_talkie_2".into()]
    }

    async fn read(&self, transport: &Transport) -> Result<UrcProfile, AdapterError> {
        let _device_id = ble_device_id(transport)?;
        // TODO Phase 3: connect via btleplug, enumerate GATT, read channel data
        Err(AdapterError::Protocol("Xiaomi BLE read not yet implemented — Phase 3".into()))
    }

    async fn write(
        &self,
        transport: &Transport,
        _profile: &UrcProfile,
    ) -> Result<WriteReport, AdapterError> {
        let _device_id = ble_device_id(transport)?;
        // TODO Phase 3: encode URC channels into Xiaomi GATT write frames
        Err(AdapterError::Protocol("Xiaomi BLE write not yet implemented — Phase 3".into()))
    }

    async fn backup(&self, transport: &Transport) -> Result<BackupImage, AdapterError> {
        let _device_id = ble_device_id(transport)?;
        // TODO Phase 3: read full configuration as backup
        Err(AdapterError::Protocol("Xiaomi BLE backup not yet implemented — Phase 3".into()))
    }

    async fn dry_run(
        &self,
        _transport: &Transport,
        profile: &UrcProfile,
    ) -> Result<DryRunReport, AdapterError> {
        Ok(urm_core::dry_run_from_core_validation(profile))
    }
}

fn ble_device_id(transport: &Transport) -> Result<String, AdapterError> {
    match transport {
        Transport::Ble { device_id } => Ok(device_id.clone()),
        _ => Err(AdapterError::Transport(
            "Xiaomi adapter requires BLE transport".into(),
        )),
    }
}
