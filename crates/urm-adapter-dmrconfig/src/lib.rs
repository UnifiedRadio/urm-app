// Phase 6 placeholder — dmrconfig bridge for headless DMR CLI automation
// dmrconfig is BSD-3-Clause — most permissive of all bridge dependencies
// Reference: https://github.com/OpenRTX/dmrconfig

use async_trait::async_trait;
use urm_core::adapter::{AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport};
use urm_core::schema::{DeviceModel, UrcProfile};

pub struct DmrconfigAdapter;

#[async_trait]
impl RadioAdapter for DmrconfigAdapter {
    fn name(&self) -> &str { "dmrconfig" }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec!["radioddity_gd77".into(), "tyt_md380".into()]
    }

    async fn read(&self, _t: &Transport) -> Result<UrcProfile, AdapterError> {
        Err(AdapterError::Protocol("dmrconfig adapter — Phase 6 not yet implemented".into()))
    }
    async fn write(&self, _t: &Transport, _p: &UrcProfile) -> Result<WriteReport, AdapterError> {
        Err(AdapterError::Protocol("dmrconfig adapter — Phase 6 not yet implemented".into()))
    }
    async fn backup(&self, _t: &Transport) -> Result<BackupImage, AdapterError> {
        Err(AdapterError::Protocol("dmrconfig adapter — Phase 6 not yet implemented".into()))
    }
    async fn dry_run(&self, _t: &Transport, _p: &UrcProfile) -> Result<DryRunReport, AdapterError> {
        Err(AdapterError::Protocol("dmrconfig adapter — Phase 6 not yet implemented".into()))
    }
}
