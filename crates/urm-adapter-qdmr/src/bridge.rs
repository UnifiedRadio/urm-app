use async_trait::async_trait;
use urm_core::adapter::{
    AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport,
};
use urm_core::schema::{DeviceModel, UrcProfile};

use crate::converter;
use crate::qdmr_env::QdmrEnv;

pub struct QdmrAdapter {
    env: QdmrEnv,
    model_id: String,
}

impl QdmrAdapter {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self { env: QdmrEnv::detect(), model_id: model_id.into() }
    }
}

#[async_trait]
impl RadioAdapter for QdmrAdapter {
    fn name(&self) -> &str {
        "qdmr"
    }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec![self.model_id.clone()]
    }

    async fn read(&self, transport: &Transport) -> Result<UrcProfile, AdapterError> {
        let port = usb_port(transport)?;
        let tmp = tempfile::NamedTempFile::new()
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        let out_path = tmp.path().to_str().unwrap().to_string();

        self.env
            .run(&["--read", "--output", &out_path, "--port", &port])
            .await
            .map_err(|e| AdapterError::Protocol(e.to_string()))?;

        let yaml = tokio::fs::read_to_string(tmp.path())
            .await
            .map_err(|e| AdapterError::Transport(e.to_string()))?;

        converter::qdmr_yaml_to_urc(&yaml, &self.model_id)
            .map_err(|e| AdapterError::Protocol(e.to_string()))
    }

    async fn write(
        &self,
        transport: &Transport,
        profile: &UrcProfile,
    ) -> Result<WriteReport, AdapterError> {
        let port = usb_port(transport)?;
        let yaml = converter::urc_to_qdmr_yaml(profile)
            .map_err(|e| AdapterError::Protocol(e.to_string()))?;

        let tmp = tempfile::NamedTempFile::new()
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        tokio::fs::write(tmp.path(), yaml.as_bytes())
            .await
            .map_err(|e| AdapterError::Transport(e.to_string()))?;

        self.env
            .run_to_file(&["--write", tmp.path().to_str().unwrap(), "--port", &port])
            .await
            .map_err(|e| AdapterError::Protocol(e.to_string()))?;

        Ok(WriteReport {
            success: true,
            channels_written: profile.channels.len(),
            warnings: vec![],
        })
    }

    async fn backup(&self, transport: &Transport) -> Result<BackupImage, AdapterError> {
        let port = usb_port(transport)?;
        let tmp = tempfile::NamedTempFile::new()
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        let out_path = tmp.path().to_str().unwrap().to_string();

        self.env
            .run(&["--read", "--output", &out_path, "--port", &port])
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;

        let data = tokio::fs::read(tmp.path())
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;

        Ok(BackupImage {
            data,
            format: "qdmr_yaml".into(),
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

fn usb_port(transport: &Transport) -> Result<String, AdapterError> {
    match transport {
        Transport::Serial { port, .. } => Ok(port.clone()),
        _ => Err(AdapterError::Transport(
            "qDMR adapter requires serial transport".into(),
        )),
    }
}
