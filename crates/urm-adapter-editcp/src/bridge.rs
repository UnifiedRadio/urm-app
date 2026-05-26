use async_trait::async_trait;
use urm_core::adapter::{
    AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport,
};
use urm_core::schema::{DeviceModel, UrcProfile};
use crate::editcp_env::EditcpEnv;

pub struct EditcpAdapter {
    env: EditcpEnv,
    model_id: String,
}

impl EditcpAdapter {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self { env: EditcpEnv::detect(), model_id: model_id.into() }
    }
}

#[async_trait]
impl RadioAdapter for EditcpAdapter {
    fn name(&self) -> &str { "editcp" }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec![self.model_id.clone()]
    }

    async fn read(&self, transport: &Transport) -> Result<UrcProfile, AdapterError> {
        let port = serial_port(transport)?;
        let tmp = tempfile::Builder::new()
            .suffix(".codeplug")
            .tempfile()
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        let out = tmp.path().to_str().unwrap().to_string();

        self.env
            .run(&["read", "--serial", &port, "--output", &out])
            .await
            .map_err(|e| AdapterError::Protocol(e.to_string()))?;

        let raw = tokio::fs::read(tmp.path())
            .await
            .map_err(|e| AdapterError::Transport(e.to_string()))?;

        // editcp writes a proprietary binary codeplug; wrap as minimal profile
        let mut profile = UrcProfile::new(&self.model_id);
        profile.devices.push(urm_core::schema::DeviceBinding {
            model: self.model_id.clone(),
            transport: "serial".into(),
            adapter: "editcp".into(),
        });
        drop(raw); // raw codeplug preserved in backup; conversion not yet implemented
        Ok(profile)
    }

    async fn write(
        &self,
        transport: &Transport,
        _profile: &UrcProfile,
    ) -> Result<WriteReport, AdapterError> {
        let _port = serial_port(transport)?;
        Err(AdapterError::Protocol(
            "editcp write: URC→editcp codeplug conversion not yet implemented".into(),
        ))
    }

    async fn backup(&self, transport: &Transport) -> Result<BackupImage, AdapterError> {
        let port = serial_port(transport)?;
        let tmp = tempfile::Builder::new()
            .suffix(".codeplug")
            .tempfile()
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;
        let out = tmp.path().to_str().unwrap().to_string();

        self.env
            .run(&["read", "--serial", &port, "--output", &out])
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;

        let data = tokio::fs::read(tmp.path())
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;

        Ok(BackupImage {
            data,
            format: "editcp_codeplug".into(),
            device_model: self.model_id.clone(),
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

fn serial_port(transport: &Transport) -> Result<String, AdapterError> {
    match transport {
        Transport::Serial { port, .. } => Ok(port.clone()),
        _ => Err(AdapterError::Transport("editcp adapter requires serial transport".into())),
    }
}
