use async_trait::async_trait;
use urm_core::adapter::{
    AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport,
};
use urm_core::schema::{DeviceModel, UrcProfile};
use crate::converter;
use crate::python_env::PythonEnv;

pub struct ChirpAdapter {
    python: PythonEnv,
    /// Internal model ID (e.g. "baofeng_uv5r") set at registration time.
    model_id: String,
    chirp_vendor: String,
    chirp_model: String,
}

impl ChirpAdapter {
    pub fn new(
        model_id: impl Into<String>,
        chirp_vendor: impl Into<String>,
        chirp_model: impl Into<String>,
    ) -> Self {
        Self {
            python: PythonEnv::detect(),
            model_id: model_id.into(),
            chirp_vendor: chirp_vendor.into(),
            chirp_model: chirp_model.into(),
        }
    }

    fn chirp_model_label(&self) -> String {
        // Existing CHIRP invocation path expects a single model string.
        format!("{} {}", self.chirp_vendor, self.chirp_model)
    }
}

#[async_trait]
impl RadioAdapter for ChirpAdapter {
    fn name(&self) -> &str {
        "chirp"
    }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec![self.model_id.clone()]
    }

    async fn read(&self, transport: &Transport) -> Result<UrcProfile, AdapterError> {
        let port = serial_port(transport)?;
        let tmp = tempfile::NamedTempFile::new()
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        let model_label = self.chirp_model_label();

        self.python
            .run_chirp(&[
                "--port", &port,
                "--model", &model_label,
                "--download", tmp.path().to_str().unwrap(),
            ])
            .await
            .map_err(|e| AdapterError::Protocol(e.to_string()))?;

        let raw = tokio::fs::read(tmp.path())
            .await
            .map_err(|e| AdapterError::Transport(e.to_string()))?;

        converter::chirp_img_to_urc(&raw)
            .map_err(|e| AdapterError::Protocol(e.to_string()))
    }

    async fn write(
        &self,
        transport: &Transport,
        profile: &UrcProfile,
    ) -> Result<WriteReport, AdapterError> {
        let port = serial_port(transport)?;
        let csv = converter::urc_to_chirp_csv(profile)
            .map_err(|e| AdapterError::Protocol(e.to_string()))?;

        let tmp = tempfile::NamedTempFile::new()
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        tokio::fs::write(tmp.path(), csv.as_bytes())
            .await
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        let model_label = self.chirp_model_label();

        self.python
            .run_chirp(&[
                "--port", &port,
                "--model", &model_label,
                "--upload", tmp.path().to_str().unwrap(),
            ])
            .await
            .map_err(|e| AdapterError::Protocol(e.to_string()))?;

        Ok(WriteReport {
            success: true,
            channels_written: profile.channels.len(),
            warnings: vec![],
        })
    }

    async fn backup(&self, transport: &Transport) -> Result<BackupImage, AdapterError> {
        let port = serial_port(transport)?;
        let tmp = tempfile::NamedTempFile::new()
            .map_err(|e| AdapterError::Transport(e.to_string()))?;
        let model_label = self.chirp_model_label();

        self.python
            .run_chirp(&[
                "--port", &port,
                "--model", &model_label,
                "--download", tmp.path().to_str().unwrap(),
            ])
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;

        let data = tokio::fs::read(tmp.path())
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;

        Ok(BackupImage {
            data,
            format: "img".into(),
            device_model: self.model_id.clone(), // accurate, not hardcoded
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
        _ => Err(AdapterError::Transport(
            "CHIRP adapter requires serial transport".into(),
        )),
    }
}
