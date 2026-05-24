use serde::{Deserialize, Serialize};
use super::channel::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrcProfile {
    pub urc_version: String,
    pub id: String,
    pub name: String,
    pub region: Option<String>,
    pub owner_type: OwnerType,
    pub compliance_mode: bool,
    pub channels: Vec<Channel>,
    pub devices: Vec<DeviceBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerType {
    Personal,
    Team,
    Activity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBinding {
    pub model: String,
    pub transport: String,
    pub adapter: String,
}

impl UrcProfile {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            urc_version: "1.0".into(),
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            region: None,
            owner_type: OwnerType::Personal,
            compliance_mode: false,
            channels: vec![],
            devices: vec![],
        }
    }
}
