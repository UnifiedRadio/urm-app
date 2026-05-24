// Phase 6 placeholder — qDMR bridge for DMR digital radios (TYT, Radioddity, AnyTone)
// Activate when Phase 1-2 are stable and DMR support is prioritized.
//
// qDMR is GPL-3.0-or-later. Call via `qdmr` CLI subprocess, same pattern as CHIRP.
// Reference: https://github.com/hmatuschek/qdmr

use async_trait::async_trait;
use urm_core::adapter::{AdapterError, BackupImage, DryRunReport, RadioAdapter, Transport, WriteReport};
use urm_core::schema::{DeviceModel, UrcProfile};

pub struct QdmrAdapter;

#[async_trait]
impl RadioAdapter for QdmrAdapter {
    fn name(&self) -> &str { "qdmr" }

    fn supported_models(&self) -> Vec<DeviceModel> {
        vec![
            "tyt_md380".into(),
            "tyt_md390".into(),
            "radioddity_gd77".into(),
            "anytone_878".into(),
        ]
    }

    async fn read(&self, _t: &Transport) -> Result<UrcProfile, AdapterError> {
        Err(AdapterError::Protocol("qDMR adapter — Phase 6 not yet implemented".into()))
    }
    async fn write(&self, _t: &Transport, _p: &UrcProfile) -> Result<WriteReport, AdapterError> {
        Err(AdapterError::Protocol("qDMR adapter — Phase 6 not yet implemented".into()))
    }
    async fn backup(&self, _t: &Transport) -> Result<BackupImage, AdapterError> {
        Err(AdapterError::Protocol("qDMR adapter — Phase 6 not yet implemented".into()))
    }
    async fn dry_run(&self, _t: &Transport, _p: &UrcProfile) -> Result<DryRunReport, AdapterError> {
        Err(AdapterError::Protocol("qDMR adapter — Phase 6 not yet implemented".into()))
    }
}
