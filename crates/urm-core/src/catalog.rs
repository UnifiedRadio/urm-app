use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

static DEVICE_CATALOG: OnceLock<DeviceCatalog> = OnceLock::new();

const DEVICES_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../urc-schema/data/devices.json"
));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCatalog {
    pub schema_version: String,
    pub generated_at: String,
    pub source: String,
    pub total_models: usize,
    pub models: Vec<DeviceCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCatalogEntry {
    pub urm_id: String,
    pub chirp_driver: Option<String>,
    pub vendor: String,
    pub model: String,
    pub aliases: Vec<String>,
    pub freq_ranges_mhz: Vec<(f64, f64)>,
    pub max_channels: usize,
    pub max_name_chars: usize,
    pub valid_modes: Vec<String>,
    pub transport: String,
    pub baud_rate: Option<u32>,
    pub adapter: Option<String>,
}

impl DeviceCatalog {
    pub fn find_by_model_or_alias(&self, model: &str) -> Option<&DeviceCatalogEntry> {
        self.models.iter().find(|entry| {
            entry.urm_id.eq_ignore_ascii_case(model)
                || entry
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(model))
        })
    }

    pub fn serial_chirp_models(&self) -> impl Iterator<Item = &DeviceCatalogEntry> {
        self.models.iter().filter(|entry| {
            entry.transport.eq_ignore_ascii_case("serial") && entry.chirp_driver.is_some()
        })
    }
}

pub fn device_catalog() -> &'static DeviceCatalog {
    DEVICE_CATALOG.get_or_init(|| {
        serde_json::from_str(DEVICES_JSON).expect("failed to parse embedded urc-schema/data/devices.json")
    })
}

pub fn canonical_model_id(model: &str) -> Option<String> {
    device_catalog()
        .find_by_model_or_alias(model)
        .map(|entry| entry.urm_id.clone())
}

