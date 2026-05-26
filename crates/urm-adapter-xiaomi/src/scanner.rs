use async_trait::async_trait;
use urm_core::adapter::{
    AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport,
};
use urm_core::schema::{DeviceModel, UrcProfile};

use crate::client::{self, BleSession};

pub struct XiaomiAdapter;

#[async_trait]
impl RadioAdapter for XiaomiAdapter {
    fn name(&self) -> &str {
        "xiaomi_ble"
    }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec![
            "xiaomi_walkie_talkie_2".into(),
            "jifeng_a108plus".into(),
        ]
    }

    async fn read(&self, transport: &Transport) -> Result<UrcProfile, AdapterError> {
        let device_id = ble_device_id(transport)?;
        let mut session = BleSession::connect(&device_id).await.map_err(proto_err)?;
        let profile = session.read_profile(&device_id).await.map_err(proto_err)?;
        session.disconnect().await.ok();
        Ok(profile)
    }

    async fn write(
        &self,
        transport: &Transport,
        profile: &UrcProfile,
    ) -> Result<WriteReport, AdapterError> {
        let device_id = ble_device_id(transport)?;
        let mut session = BleSession::connect(&device_id).await.map_err(proto_err)?;

        let params = client::urc_to_write_params(profile);
        let total = params.len();
        let mut written = 0;
        let mut warnings = Vec::new();

        for (seq, rx_hz, tx_hz, rx_css, tx_css) in params {
            match session.write_channel(seq, rx_hz, tx_hz, rx_css, tx_css).await {
                Ok(()) => written += 1,
                Err(e) => warnings.push(format!("channel {seq}: {e}")),
            }
        }

        session.disconnect().await.ok();

        if written == 0 && total > 0 {
            return Err(AdapterError::WriteAborted(format!(
                "no channels written; errors: {}",
                warnings.join("; ")
            )));
        }

        Ok(WriteReport {
            success: warnings.is_empty(),
            channels_written: written,
            warnings,
        })
    }

    async fn backup(&self, transport: &Transport) -> Result<BackupImage, AdapterError> {
        let device_id = ble_device_id(transport)?;
        let mut session = BleSession::connect(&device_id).await.map_err(proto_err)?;
        let profile = session.read_profile(&device_id).await.map_err(proto_err)?;
        session.disconnect().await.ok();

        let data = serde_json::to_vec(&profile)
            .map_err(|e| AdapterError::Protocol(format!("backup serialisation: {e}")))?;

        Ok(BackupImage {
            data,
            format: "xiaomi_json".into(),
            device_model: "xiaomi_walkie_talkie_2".into(),
        })
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

fn proto_err(e: anyhow::Error) -> AdapterError {
    AdapterError::Protocol(e.to_string())
}
