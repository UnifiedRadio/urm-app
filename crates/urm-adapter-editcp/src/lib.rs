// Phase 6 placeholder — editcp bridge for MD-380/MD-390/Alinco DJ-MD40
// Reference: https://github.com/DaleFarnsworth-DMR/editcp

use async_trait::async_trait;
use urm_core::adapter::{AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport};
use urm_core::schema::{DeviceModel, UrcProfile};

pub struct EditcpAdapter;

#[async_trait]
impl RadioAdapter for EditcpAdapter {
    fn name(&self) -> &str { "editcp" }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec!["tyt_md380".into(), "tyt_md390".into(), "alinco_dj_md40".into()]
    }

    async fn read(&self, _t: &Transport) -> Result<UrcProfile, AdapterError> {
        Err(AdapterError::Protocol("editcp adapter — Phase 6 not yet implemented".into()))
    }
    async fn write(&self, _t: &Transport, _p: &UrcProfile) -> Result<WriteReport, AdapterError> {
        Err(AdapterError::Protocol("editcp adapter — Phase 6 not yet implemented".into()))
    }
    async fn backup(&self, _t: &Transport) -> Result<BackupImage, AdapterError> {
        Err(AdapterError::Protocol("editcp adapter — Phase 6 not yet implemented".into()))
    }
    async fn dry_run(&self, _t: &Transport, _p: &UrcProfile) -> Result<DryRunReport, AdapterError> {
        Err(AdapterError::Protocol("editcp adapter — Phase 6 not yet implemented".into()))
    }
}
