use urm_core::schema::{
    Channel, ChannelMode, AnalogFields, UrcProfile,
    PowerLevel, Bandwidth, ToneType,
};

// ── CHIRP CSV text import ─────────────────────────────────────────────────────

/// Parse a CHIRP-exported text CSV into a URC-v1 profile.
/// Handles the standard CHIRP column layout; tolerates missing optional columns.
pub fn chirp_csv_to_urc(text: &str) -> anyhow::Result<UrcProfile> {
    let mut lines = text.lines().peekable();

    // Skip any leading comment lines (CHIRP sometimes prepends them)
    while let Some(&line) = lines.peek() {
        if line.trim_start().starts_with('#') {
            lines.next();
        } else {
            break;
        }
    }

    let header = lines.next().ok_or_else(|| anyhow::anyhow!("empty CSV file"))?;

    // Build column index map — case-insensitive, trims BOM and whitespace
    let cols: Vec<String> = header
        .trim_start_matches('\u{feff}') // strip UTF-8 BOM if present
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect();

    let col = |name: &str| -> usize {
        cols.iter()
            .position(|c| c == name)
            .unwrap_or(usize::MAX)
    };

    let idx_name = col("name");
    let idx_freq = col("frequency");
    let idx_duplex = col("duplex");
    let idx_offset = col("offset");
    let idx_tone = col("tone");
    let idx_rtone = col("rtonefreq");
    let idx_ctone = col("ctonefreq");
    let idx_dtcs = col("dtcscode");
    let idx_mode = col("mode");
    let idx_skip = col("skip");

    if idx_freq == usize::MAX {
        anyhow::bail!("CSV does not contain a 'Frequency' column — is this a CHIRP export?");
    }

    let mut channels = Vec::new();

    for raw_line in lines {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        let get = |idx: usize| -> &str {
            if idx == usize::MAX { return ""; }
            fields.get(idx).map(|s| s.trim()).unwrap_or("")
        };

        // Skip empty/unused channel slots
        let freq_str = get(idx_freq);
        if freq_str.is_empty() {
            continue;
        }
        let rx_freq: f64 = match freq_str.parse() {
            Ok(f) if f > 0.0 => f,
            _ => continue,
        };

        let name = get(idx_name);
        if name.is_empty() {
            continue;
        }

        // TX frequency from duplex + offset
        let offset: f64 = get(idx_offset).parse().unwrap_or(0.0);
        let tx_freq = match get(idx_duplex) {
            "+" => rx_freq + offset,
            "-" => rx_freq - offset,
            "split" | "Split" => {
                if offset > 0.0 { offset } else { rx_freq }
            }
            _ => rx_freq,
        };

        // Tone / squelch
        let tone_mode = get(idx_tone);
        let rtone_hz: f64 = get(idx_rtone).parse().unwrap_or(88.5);
        let ctone_hz: f64 = get(idx_ctone).parse().unwrap_or(88.5);
        let _dtcs = get(idx_dtcs);

        let (tone_type, tone_value_hz) = match tone_mode {
            "Tone" => (ToneType::Ctcss, Some(rtone_hz)),
            "TSQL" => (ToneType::Ctcss, Some(ctone_hz)),
            "DTCS" | "DCS" | "DTCS-R" => (ToneType::Dcs, None),
            _ => (ToneType::None, None),
        };

        // Bandwidth from Mode column: NFM → narrow, FM → wide
        let bandwidth = if get(idx_mode).eq_ignore_ascii_case("NFM") {
            Bandwidth::Narrow
        } else {
            Bandwidth::Wide
        };

        // Skip flag — "S" means skip in scan
        let scan = get(idx_skip).is_empty();

        channels.push(Channel {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.chars().take(16).collect(),
            mode: ChannelMode::AnalogFm,
            rx_freq_mhz: rx_freq,
            tx_freq_mhz: tx_freq,
            power: PowerLevel::High,
            bandwidth,
            tags: vec![],
            analog: Some(AnalogFields { tone_type, tone_value_hz, scan }),
            dmr: None,
        });
    }

    if channels.is_empty() {
        anyhow::bail!("no valid channels found in CSV — all rows had zero frequency or empty names");
    }

    let mut profile = UrcProfile::new("Imported from CHIRP");
    profile.channels = channels;
    Ok(profile)
}

// ── CHIRP binary image stub ───────────────────────────────────────────────────

/// Convert a CHIRP .img binary download into a URC-v1 profile.
/// Full parse in Phase 2 — currently returns the CSV parse path if file looks
/// like text, otherwise errors with a clear message.
pub fn chirp_img_to_urc(raw: &[u8]) -> anyhow::Result<UrcProfile> {
    // If the bytes look like UTF-8 text (starts with printable ASCII), try CSV parse
    if raw.first().map(|b| b.is_ascii_graphic() || *b == b'\n' || *b == 0xEF).unwrap_or(false) {
        let text = std::str::from_utf8(raw)
            .map_err(|_| anyhow::anyhow!("file is not valid UTF-8"))?;
        return chirp_csv_to_urc(text);
    }
    anyhow::bail!(
        "binary .img format not yet supported — export your CHIRP config as CSV \
         (File → Export → CSV) and import that instead"
    )
}

// ── CHIRP CSV export ──────────────────────────────────────────────────────────

/// Serialize a URC-v1 profile into a CHIRP-compatible CSV.
/// Only analog FM channels are included; DMR/C4FM are silently skipped
/// (CHIRP only handles analog).
pub fn urc_to_chirp_csv(profile: &UrcProfile) -> anyhow::Result<String> {
    let header = "Location,Name,Frequency,Duplex,Offset,Tone,rToneFreq,cToneFreq,\
                  DtcsCode,DtcsPolarity,Mode,TStep,Skip,Comment";
    let mut rows = vec![header.to_string()];

    let mut location = 0usize;
    for ch in &profile.channels {
        if ch.mode != ChannelMode::AnalogFm {
            continue;
        }

        let offset_mhz = (ch.tx_freq_mhz - ch.rx_freq_mhz).abs();
        let duplex = if ch.tx_freq_mhz > ch.rx_freq_mhz + 0.0001 {
            "+"
        } else if ch.rx_freq_mhz > ch.tx_freq_mhz + 0.0001 {
            "-"
        } else {
            ""
        };

        let mode = match ch.bandwidth {
            Bandwidth::Narrow => "NFM",
            Bandwidth::Wide => "FM",
        };

        let (tone_mode, rtone, ctone) = analog_tone(ch);

        let skip = ch.analog.as_ref().map(|a| if a.scan { "" } else { "S" }).unwrap_or("");

        let name_truncated: String = ch.name.chars().take(7).collect();
        rows.push(format!(
            "{},{},{:.6},{},{:.6},{},{},{},023,NN,{},5.00,{},",
            location,
            name_truncated,
            ch.rx_freq_mhz,
            duplex,
            offset_mhz,
            tone_mode,
            rtone,
            ctone,
            mode,
            skip,
        ));
        location += 1;
    }

    Ok(rows.join("\n"))
}

fn analog_tone(ch: &Channel) -> (&'static str, String, String) {
    match &ch.analog {
        Some(a) => match a.tone_type {
            ToneType::Ctcss => {
                let v = format!("{:.1}", a.tone_value_hz.unwrap_or(88.5));
                ("TSQL", v.clone(), v)
            }
            ToneType::Dcs => ("DTCS", "88.5".into(), "88.5".into()),
            ToneType::None => ("", "88.5".into(), "88.5".into()),
        },
        None => ("", "88.5".into(), "88.5".into()),
    }
}
