use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub mode: ChannelMode,
    pub rx_freq_mhz: f64,
    pub tx_freq_mhz: f64,
    pub power: PowerLevel,
    pub bandwidth: Bandwidth,
    pub tags: Vec<String>,
    pub analog: Option<AnalogFields>,
    pub dmr: Option<DmrFields>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChannelMode {
    AnalogFm,
    Dmr,
    C4fm,
    Dstar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bandwidth {
    Wide,
    Narrow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogFields {
    pub tone_type: ToneType,
    pub tone_value_hz: Option<f64>,
    pub scan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToneType {
    None,
    Ctcss,
    Dcs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmrFields {
    pub color_code: u8,
    pub time_slot: u8,
    pub talkgroup_id: u32,
    pub contact_ref: String,
    pub rx_group_ref: String,
    pub zone_ref: String,
}
