use async_trait::async_trait;
use crate::schema::{UrcProfile, DeviceModel};
use crate::schema::validation::validate_profile;

/// The single extension point for all hardware backends.
/// External crates implement this trait against urm-core = { features = ["adapter-api"] }.
#[async_trait]
pub trait RadioAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn supported_models(&self) -> Vec<DeviceModel>;

    /// Read current configuration from device into URC-v1 profile.
    async fn read(&self, transport: &Transport) -> Result<UrcProfile, AdapterError>;

    /// Write URC-v1 profile to device. Must backup first — enforced by task_queue.
    async fn write(
        &self,
        transport: &Transport,
        profile: &UrcProfile,
    ) -> Result<WriteReport, AdapterError>;

    /// Read raw firmware image for backup purposes.
    async fn backup(&self, transport: &Transport) -> Result<BackupImage, AdapterError>;

    /// Validate that a profile is compatible with this adapter before write.
    async fn dry_run(
        &self,
        transport: &Transport,
        profile: &UrcProfile,
    ) -> Result<DryRunReport, AdapterError>;
}

/// Shared dry-run implementation: runs core validation and maps results into DryRunReport.
/// Adapters that don't need hardware-specific validation can delegate to this.
pub fn dry_run_from_core_validation(profile: &UrcProfile) -> DryRunReport {
    let result = validate_profile(profile);
    DryRunReport {
        valid: result.is_valid(),
        errors: result.errors.iter().map(|e| ValidationIssue {
            channel_id: e.channel_id.clone(),
            field: Some(e.field.clone()),
            message: e.message.clone(),
        }).collect(),
        warnings: result.warnings.iter().map(|w| ValidationIssue {
            channel_id: w.channel_id.clone(),
            field: Some(w.field.clone()),
            message: w.message.clone(),
        }).collect(),
    }
}

#[derive(Debug, Clone)]
pub enum Transport {
    Serial { port: String, baud_rate: u32 },
    Ble { device_id: String },
    Otg { device_path: String },
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WriteReport {
    pub success: bool,
    pub channels_written: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BackupImage {
    pub data: Vec<u8>,
    pub format: String,
    pub device_model: String,
}

#[derive(Debug, Clone)]
pub struct DryRunReport {
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub channel_id: Option<String>,
    pub field: Option<String>,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("device not found or not responding")]
    DeviceNotFound,
    #[error("model mismatch: expected {expected}, detected {detected}")]
    ModelMismatch { expected: String, detected: String },
    #[error("backup failed: {0}")]
    BackupFailed(String),
    #[error("write aborted: {0}")]
    WriteAborted(String),
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
