use super::channel::{Channel, ChannelMode};
use super::profile::UrcProfile;
use crate::catalog::device_catalog;

// ── Base validation ───────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug)]
pub struct ValidationError {
    pub channel_id: Option<String>,
    pub field: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ValidationWarning {
    pub channel_id: Option<String>,
    pub field: String,
    pub message: String,
}

pub fn validate_profile(profile: &UrcProfile) -> ValidationResult {
    let mut result = ValidationResult::default();
    let mut seen_ids = std::collections::HashSet::new();

    for ch in &profile.channels {
        if !seen_ids.insert(&ch.id) {
            result.errors.push(ValidationError {
                channel_id: Some(ch.id.clone()),
                field: "id".into(),
                message: format!("duplicate channel id '{}'", ch.id),
            });
        }
        validate_channel(ch, &mut result);
    }

    result
}

fn validate_channel(ch: &Channel, result: &mut ValidationResult) {
    if ch.name.is_empty() {
        result.errors.push(ValidationError {
            channel_id: Some(ch.id.clone()),
            field: "name".into(),
            message: "channel name must not be empty".into(),
        });
    }

    if ch.rx_freq_mhz <= 0.0 {
        result.errors.push(ValidationError {
            channel_id: Some(ch.id.clone()),
            field: "rx_freq_mhz".into(),
            message: "receive frequency must be positive".into(),
        });
    }

    if ch.mode == ChannelMode::AnalogFm && ch.analog.is_none() {
        result.warnings.push(ValidationWarning {
            channel_id: Some(ch.id.clone()),
            field: "analog".into(),
            message: "analog_fm channel has no analog fields".into(),
        });
    }

    if ch.mode == ChannelMode::Dmr && ch.dmr.is_none() {
        result.errors.push(ValidationError {
            channel_id: Some(ch.id.clone()),
            field: "dmr".into(),
            message: "dmr channel requires dmr fields".into(),
        });
    }
}

// ── Device-specific validation (FR-03 extension) ──────────────────────────────

pub struct DeviceConstraints {
    pub model: String,
    pub freq_ranges: Vec<(f64, f64)>,  // (min_mhz, max_mhz) inclusive
    pub max_channels: usize,
    pub max_name_chars: usize,
}

pub fn constraints_for_model(model: &str) -> DeviceConstraints {
    if let Some(entry) = device_catalog().find_by_model_or_alias(model) {
        let freq_ranges = if entry.freq_ranges_mhz.is_empty() {
            vec![(30.0, 1300.0)]
        } else {
            entry.freq_ranges_mhz.clone()
        };
        return DeviceConstraints {
            model: entry.urm_id.clone(),
            freq_ranges,
            max_channels: entry.max_channels,
            max_name_chars: entry.max_name_chars,
        };
    }

    // Unknown model — permissive defaults, no range restriction
    DeviceConstraints {
        model: model.to_string(),
        freq_ranges: vec![(30.0, 1300.0)],
        max_channels: 9999,
        max_name_chars: 16,
    }
}

/// Validate a profile against the constraints of a specific device model.
/// Runs base validation first, then adds device-specific checks.
pub fn validate_profile_for_device(profile: &UrcProfile, model: &str) -> ValidationResult {
    let mut result = validate_profile(profile);
    let c = constraints_for_model(model);

    // Channel count
    if profile.channels.len() > c.max_channels {
        result.errors.push(ValidationError {
            channel_id: None,
            field: "channels".into(),
            message: format!(
                "{} supports at most {} channels, but profile has {}",
                c.model, c.max_channels, profile.channels.len()
            ),
        });
    }

    for ch in &profile.channels {
        // Frequency range
        let rx_ok = c.freq_ranges.iter().any(|(lo, hi)| ch.rx_freq_mhz >= *lo && ch.rx_freq_mhz <= *hi);
        if !rx_ok && ch.rx_freq_mhz > 0.0 {
            let ranges: Vec<String> = c.freq_ranges.iter()
                .map(|(lo, hi)| format!("{lo}–{hi} MHz"))
                .collect();
            result.errors.push(ValidationError {
                channel_id: Some(ch.id.clone()),
                field: "rx_freq_mhz".into(),
                message: format!(
                    "{:.4} MHz is outside {}'s supported range ({})",
                    ch.rx_freq_mhz, c.model, ranges.join(", ")
                ),
            });
        }

        let tx_ok = c.freq_ranges.iter().any(|(lo, hi)| ch.tx_freq_mhz >= *lo && ch.tx_freq_mhz <= *hi);
        if !tx_ok && ch.tx_freq_mhz > 0.0 {
            result.errors.push(ValidationError {
                channel_id: Some(ch.id.clone()),
                field: "tx_freq_mhz".into(),
                message: format!(
                    "TX {:.4} MHz is outside {}'s supported range",
                    ch.tx_freq_mhz, c.model
                ),
            });
        }

        // Name length
        if ch.name.chars().count() > c.max_name_chars {
            result.warnings.push(ValidationWarning {
                channel_id: Some(ch.id.clone()),
                field: "name".into(),
                message: format!(
                    "'{}' is {} chars — {} hardware displays at most {} chars",
                    ch.name, ch.name.chars().count(), c.model, c.max_name_chars
                ),
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Bandwidth, Channel, ChannelMode, PowerLevel, UrcProfile};

    fn sample_channel(name: &str, rx: f64, tx: f64) -> Channel {
        Channel {
            id: "ch1".into(),
            name: name.into(),
            mode: ChannelMode::AnalogFm,
            rx_freq_mhz: rx,
            tx_freq_mhz: tx,
            power: PowerLevel::High,
            bandwidth: Bandwidth::Narrow,
            tags: vec![],
            analog: None,
            dmr: None,
        }
    }

    #[test]
    fn resolves_alias_constraints() {
        let c = constraints_for_model("baofeng_uv5ra");
        assert_eq!(c.model, "baofeng_uv5r");
        assert_eq!(c.max_channels, 128);
        assert_eq!(c.max_name_chars, 7);
    }

    #[test]
    fn validates_frequency_range_from_catalog() {
        let mut profile = UrcProfile::new("test");
        profile.channels.push(sample_channel("local", 350.0, 350.0));
        let result = validate_profile_for_device(&profile, "baofeng_uv5r");
        assert!(!result.is_valid());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field == "rx_freq_mhz" && e.message.contains("supported range"))
        );
    }

    #[test]
    fn validates_channel_count_from_catalog() {
        let mut profile = UrcProfile::new("count");
        for i in 0..17 {
            profile
                .channels
                .push(sample_channel(&format!("C{i}"), 433.0, 433.0));
        }
        let result = validate_profile_for_device(&profile, "xiaomi_wt2");
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.field == "channels" && e.message.contains("at most"))
        );
    }

    #[test]
    fn warns_for_name_length_from_catalog() {
        let mut profile = UrcProfile::new("name");
        profile.channels.push(sample_channel("ABCDEFGH", 433.0, 433.0));
        let result = validate_profile_for_device(&profile, "baofeng_uv5r");
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.field == "name" && w.message.contains("at most 7 chars"))
        );
    }
}
