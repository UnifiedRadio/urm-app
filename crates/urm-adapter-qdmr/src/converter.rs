use anyhow::Result;
use urm_core::schema::{
    AnalogFields, Bandwidth, Channel, ChannelMode, DmrFields, PowerLevel, ToneType, UrcProfile,
};

/// Parse a minimal qDMR YAML codeplug back into a UrcProfile.
pub fn qdmr_yaml_to_urc(yaml: &str, device_model: &str) -> Result<UrcProfile> {
    let doc: serde_yaml::Value = serde_yaml::from_str(yaml)?;
    let channels_val = doc
        .get("Channels")
        .and_then(|v| v.as_sequence())
        .ok_or_else(|| anyhow::anyhow!("qDMR YAML missing 'Channels' list"))?;

    let mut channels = Vec::new();
    for ch in channels_val {
        let name = ch
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Channel")
            .to_string();

        let rx_freq_mhz = parse_freq_mhz(ch.get("rxFreq")).unwrap_or(0.0);
        let tx_freq_mhz = parse_freq_mhz(ch.get("txFreq")).unwrap_or(rx_freq_mhz);

        let is_dmr = ch.get("colorCode").is_some();

        let bw_str = ch.get("bandwidth").and_then(|v| v.as_str()).unwrap_or("Narrow");
        let bandwidth = if bw_str == "Wide" { Bandwidth::Wide } else { Bandwidth::Narrow };

        let (mode, analog, dmr) = if is_dmr {
            let color_code = ch.get("colorCode").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
            let ts_str = ch.get("timeSlot").and_then(|v| v.as_str()).unwrap_or("TS1");
            let time_slot: u8 = if ts_str == "TS2" { 2 } else { 1 };
            (
                ChannelMode::Dmr,
                None,
                Some(DmrFields {
                    color_code,
                    time_slot,
                    talkgroup_id: 0,
                    contact_ref: String::new(),
                    rx_group_ref: String::new(),
                    zone_ref: String::new(),
                }),
            )
        } else {
            let (tone_type, tone_value_hz) = parse_tone(ch.get("txTone"));
            (
                ChannelMode::AnalogFm,
                Some(AnalogFields { tone_type, tone_value_hz, scan: false }),
                None,
            )
        };

        channels.push(Channel {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            mode,
            rx_freq_mhz,
            tx_freq_mhz,
            power: PowerLevel::High,
            bandwidth,
            tags: vec![],
            analog,
            dmr,
        });
    }

    let mut profile = UrcProfile::new(device_model);
    profile.channels = channels;
    Ok(profile)
}

/// Serialize a UrcProfile to a minimal qDMR YAML codeplug.
pub fn urc_to_qdmr_yaml(profile: &UrcProfile) -> Result<String> {
    use serde_yaml::Value;

    let mut channel_list = Vec::new();

    for ch in &profile.channels {
        let mut map = serde_yaml::Mapping::new();
        map.insert(val("name"), val(&ch.name));
        map.insert(val("rxFreq"), val(mhz_to_str(ch.rx_freq_mhz)));
        map.insert(val("txFreq"), val(mhz_to_str(ch.tx_freq_mhz)));
        map.insert(val("power"), val("High"));

        match ch.mode {
            ChannelMode::Dmr => {
                if let Some(dmr) = &ch.dmr {
                    map.insert(val("colorCode"), Value::Number(dmr.color_code.into()));
                    map.insert(
                        val("timeSlot"),
                        val(if dmr.time_slot == 2 { "TS2" } else { "TS1" }),
                    );
                } else {
                    map.insert(val("colorCode"), Value::Number(1u64.into()));
                    map.insert(val("timeSlot"), val("TS1"));
                }
                map.insert(val("rxGroupList"), Value::Null);
                map.insert(val("txContact"), Value::Null);
            }
            _ => {
                let bw = match ch.bandwidth {
                    Bandwidth::Wide => "Wide",
                    Bandwidth::Narrow => "Narrow",
                };
                map.insert(val("bandwidth"), val(bw));
                if let Some(analog) = &ch.analog {
                    let tone = tone_to_val(analog.tone_type.clone(), analog.tone_value_hz);
                    map.insert(val("rxTone"), tone.clone());
                    map.insert(val("txTone"), tone);
                } else {
                    map.insert(val("rxTone"), Value::Null);
                    map.insert(val("txTone"), Value::Null);
                }
            }
        }

        channel_list.push(Value::Mapping(map));
    }

    let mut root = serde_yaml::Mapping::new();
    root.insert(val("Channels"), Value::Sequence(channel_list));
    Ok(serde_yaml::to_string(&Value::Mapping(root))?)
}

fn val(s: impl Into<String>) -> serde_yaml::Value {
    serde_yaml::Value::String(s.into())
}

fn parse_freq_mhz(v: Option<&serde_yaml::Value>) -> Option<f64> {
    let s = v?.as_str()?;
    let s = s.trim_end_matches("MHz").trim();
    s.parse().ok()
}

fn mhz_to_str(mhz: f64) -> String {
    format!("{:.4}MHz", mhz)
}

fn parse_tone(v: Option<&serde_yaml::Value>) -> (ToneType, Option<f64>) {
    let Some(s) = v.and_then(|v| v.as_str()) else {
        return (ToneType::None, None);
    };
    if s == "null" || s.is_empty() {
        return (ToneType::None, None);
    }
    if let Some(dcs) = s.strip_prefix("DCS") {
        let code: f64 = dcs.parse().unwrap_or(0.0);
        return (ToneType::Dcs, Some(code));
    }
    let hz: f64 = s.trim_end_matches("Hz").trim().parse().unwrap_or(0.0);
    (ToneType::Ctcss, Some(hz))
}

fn tone_to_val(tone_type: ToneType, value_hz: Option<f64>) -> serde_yaml::Value {
    match tone_type {
        ToneType::None => serde_yaml::Value::Null,
        ToneType::Dcs => val(format!("DCS{:03.0}", value_hz.unwrap_or(0.0))),
        ToneType::Ctcss => val(format!("{:.1}Hz", value_hz.unwrap_or(0.0))),
    }
}
