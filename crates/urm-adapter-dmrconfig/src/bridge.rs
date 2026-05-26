use async_trait::async_trait;
use urm_core::adapter::{
    AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport,
};
use urm_core::schema::{DeviceModel, UrcProfile};
use crate::dmrconfig_env::DmrconfigEnv;

pub struct DmrconfigAdapter {
    env: DmrconfigEnv,
    model_id: String,
}

impl DmrconfigAdapter {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self { env: DmrconfigEnv::detect(), model_id: model_id.into() }
    }
}

#[async_trait]
impl RadioAdapter for DmrconfigAdapter {
    fn name(&self) -> &str { "dmrconfig" }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec![self.model_id.clone()]
    }

    async fn read(&self, transport: &Transport) -> Result<UrcProfile, AdapterError> {
        let port = serial_port(transport)?;
        let tmp = tempfile::Builder::new()
            .suffix(".bin")
            .tempfile()
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        let out = tmp.path().to_str().unwrap().to_string();

        // dmrconfig -r reads from radio to file
        self.env
            .run(&["-r", &out, &port])
            .await
            .map_err(|e| AdapterError::Protocol(e.to_string()))?;

        // Return minimal profile; binary image preserved in backup
        let mut profile = UrcProfile::new(&self.model_id);
        profile.devices.push(urm_core::schema::DeviceBinding {
            model: self.model_id.clone(),
            transport: "serial".into(),
            adapter: "dmrconfig".into(),
        });
        Ok(profile)
    }

    async fn write(
        &self,
        transport: &Transport,
        _profile: &UrcProfile,
    ) -> Result<WriteReport, AdapterError> {
        let _port = serial_port(transport)?;
        Err(AdapterError::Protocol(
            "dmrconfig write: URC→dmrconfig binary conversion not yet implemented".into(),
        ))
    }

    async fn backup(&self, transport: &Transport) -> Result<BackupImage, AdapterError> {
        let port = serial_port(transport)?;
        let tmp = tempfile::Builder::new()
            .suffix(".bin")
            .tempfile()
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;
        let out = tmp.path().to_str().unwrap().to_string();

        self.env
            .run(&["-r", &out, &port])
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;

        let data = tokio::fs::read(tmp.path())
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;

        Ok(BackupImage {
            data,
            format: "dmrconfig_bin".into(),
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
        _ => Err(AdapterError::Transport("dmrconfig adapter requires serial transport".into())),
    }
}
