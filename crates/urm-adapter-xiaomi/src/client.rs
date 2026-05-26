// Full BLE session management for the Xiaomi/极蜂 Lite protocol.
//
// Flow: scan → connect → MTU request → subscribe → ConnectRequest →
//       handle auth (plain or ECDH) → DevRequest → channel read/write → disconnect.
//
// Hardware note: BLE connectivity is implemented per the documented protocol.
// Correctness on real devices requires hardware validation (Phase 3 follow-up).

use std::pin::Pin;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, PeripheralProperties, ScanFilter,
    ValueNotification, WriteType,
};
use btleplug::platform::{Manager, Peripheral};
use futures::{Stream, StreamExt};
use tracing::{debug, info, warn};
use urm_core::schema::{
    AnalogFields, Bandwidth, Channel, ChannelMode, DeviceBinding, PowerLevel, ToneType, UrcProfile,
};

use crate::crypto::EcdhKeypair;
use crate::gatt;
use crate::protocol::{
    self, ConnectResult, RawChannelInfo, CMD_CHANNEL_READ, CMD_CHANNEL_WRITE,
    CMD_CONNECT_REQUEST, CMD_DEV_REQUEST,
};

// ── Constants ─────────────────────────────────────────────────────────────────

const CHANNEL_COUNT: u32 = 16;
const SCAN_DURATION: Duration = Duration::from_secs(3);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);
const AUTH_WAIT_TIMEOUT: Duration = Duration::from_secs(20);

// ── Scanned device info ───────────────────────────────────────────────────────

pub struct ScannedDevice {
    pub id: String,
    pub name: Option<String>,
    pub rssi: Option<i16>,
}

/// Scan for Xiaomi/极蜂 BLE devices (filtered by FDAB service UUID).
pub async fn scan_xiaomi_devices() -> Result<Vec<ScannedDevice>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no BLE adapter found"))?;

    let filter = ScanFilter {
        services: vec![gatt::service_uuid()],
    };
    adapter.start_scan(filter).await?;
    tokio::time::sleep(SCAN_DURATION).await;
    adapter.stop_scan().await?;

    let peripherals = adapter.peripherals().await?;
    let mut result = Vec::new();
    for p in peripherals {
        let props: Option<PeripheralProperties> = p.properties().await.ok().flatten();
        result.push(ScannedDevice {
            id: p.id().to_string(),
            name: props.as_ref().and_then(|p| p.local_name.clone()),
            rssi: props.as_ref().and_then(|p| p.rssi),
        });
    }
    Ok(result)
}

// ── Session state ─────────────────────────────────────────────────────────────

#[derive(Default)]
struct SessionKeys {
    aes: Option<crate::crypto::SessionKey>,
    token: Vec<u8>,
}

impl SessionKeys {
    fn encrypt(&self, payload: &[u8]) -> Vec<u8> {
        match &self.aes {
            Some(k) => k.encrypt(payload),
            None => payload.to_vec(),
        }
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        match &self.aes {
            Some(k) => k.decrypt(data),
            None => Ok(data.to_vec()),
        }
    }
}

// ── BLE session ───────────────────────────────────────────────────────────────

pub struct BleSession {
    peripheral: Peripheral,
    write_char: Characteristic,
    stream: Pin<Box<dyn Stream<Item = ValueNotification> + Send>>,
    keys: SessionKeys,
}

impl BleSession {
    /// Connect, discover GATT, subscribe, and complete the auth handshake.
    pub async fn connect(device_id: &str) -> Result<Self> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let adapter = adapters
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no BLE adapter found"))?;

        // Re-scan to locate the device (it may not be cached).
        let filter = ScanFilter {
            services: vec![gatt::service_uuid()],
        };
        adapter.start_scan(filter).await?;
        tokio::time::sleep(SCAN_DURATION).await;
        adapter.stop_scan().await?;

        let peripherals = adapter.peripherals().await?;
        let peripheral = peripherals
            .into_iter()
            .find(|p| p.id().to_string() == device_id)
            .ok_or_else(|| anyhow!("BLE device not found after scan: {device_id}"))?;

        info!("connecting to BLE device {device_id}");
        peripheral.connect().await?;
        peripheral.discover_services().await?;

        let chars = peripheral.characteristics();

        let write_char = chars
            .iter()
            .find(|c| c.uuid == gatt::write_char_uuid())
            .cloned()
            .ok_or_else(|| anyhow!("write characteristic {} not found", gatt::write_char_uuid()))?;

        let notify_char = chars
            .iter()
            .find(|c| c.uuid == gatt::notify_char_uuid())
            .cloned()
            .ok_or_else(|| {
                anyhow!("notify characteristic {} not found", gatt::notify_char_uuid())
            })?;

        // Subscribe before opening the notification stream so we don't miss the auth reply.
        peripheral.subscribe(&notify_char).await?;
        let stream = peripheral.notifications().await?;

        let mut session = Self {
            peripheral,
            write_char,
            stream,
            keys: SessionKeys::default(),
        };

        session.authenticate().await?;
        Ok(session)
    }

    // ── Authentication handshake ──────────────────────────────────────────────

    async fn authenticate(&mut self) -> Result<()> {
        // Generate an ECDH keypair; the public key goes into ConnectRequest field 5.
        let ecdh = EcdhKeypair::generate();
        let pub_key = ecdh.public_key_bytes.clone();

        let connect_payload = protocol::encode_connect_request(0, &pub_key);
        let connect_frame = protocol::build_frame(CMD_CONNECT_REQUEST, &connect_payload);
        self.write_raw(&connect_frame).await?;

        // Wait for ConnectResponse (20002) or LoginResponse (20024).
        let resp = self.wait_frame(AUTH_WAIT_TIMEOUT).await?;

        match resp.cmd_id {
            protocol::CMD_CONNECT_RESPONSE => {
                let cr = protocol::parse_connect_response(&resp.payload)?;
                match cr.result {
                    ConnectResult::Allow => {
                        debug!("auth: ALLOW (unencrypted session)");
                        // ecdh keypair is no longer needed
                    }
                    ConnectResult::Wait => {
                        info!("auth: WAIT — device waiting for user confirmation (up to 20 s)");
                        let confirm = self.wait_frame(AUTH_WAIT_TIMEOUT).await?;
                        let cr2 = protocol::parse_connect_response(&confirm.payload)?;
                        if cr2.result != ConnectResult::Allow {
                            bail!("device refused after wait: {:?}", cr2.result);
                        }
                        debug!("auth: confirmed after wait");
                    }
                    other => bail!("device refused connection: {:?}", other),
                }
            }
            20024 => {
                // LoginResponse: device sends its public key + encrypted session data.
                let lr = protocol::parse_connect_response(&resp.payload)?;
                if lr.result != ConnectResult::Allow {
                    bail!("LoginResponse: not ALLOW ({:?})", lr.result);
                }
                match (lr.device_public_key, lr.encrypted_session_data) {
                    (Some(dev_pubkey), Some(enc_data)) => {
                        let transport_key = ecdh.derive_transport_key(&dev_pubkey)?;
                        let (session_key, token) = transport_key.decrypt_session(&enc_data)?;
                        self.keys.aes = Some(session_key);
                        self.keys.token = token;
                        info!("auth: ECDH session key established");
                    }
                    _ => {
                        warn!("LoginResponse missing pubkey or encrypted payload — continuing without session encryption");
                    }
                }
            }
            other => bail!("unexpected auth response cmdId: {other}"),
        }

        // Send DevRequest (cmdId 20003) to complete the handshake.
        // Payload: { version: 1 } — proto3 varint field 1 = 1
        let dev_payload = self.keys.encrypt(&[0x08, 0x01]);
        let dev_frame = protocol::build_frame(CMD_DEV_REQUEST, &dev_payload);
        self.write_raw(&dev_frame).await?;
        // Drain the DevResponse — we don't parse it, just consume it.
        let _ = self.wait_frame(RESPONSE_TIMEOUT).await;

        Ok(())
    }

    // ── Channel operations ────────────────────────────────────────────────────

    /// Read all channels (1..=CHANNEL_COUNT). Stops on first error/empty response.
    pub async fn read_all_channels(&mut self) -> Result<Vec<RawChannelInfo>> {
        let mut channels = Vec::new();
        for seq in 1..=CHANNEL_COUNT {
            let payload = protocol::encode_channel_read_request(seq);
            match self.send_recv(CMD_CHANNEL_READ, &payload).await {
                Ok(frame) => {
                    let dec = self.keys.decrypt(&frame.payload)?;
                    match protocol::decode_channel_info_request(&dec) {
                        Ok(ch) => {
                            debug!("channel {seq}: rx={}Hz tx={}Hz", ch.rx_hz, ch.tx_hz);
                            channels.push(ch);
                        }
                        Err(e) => {
                            debug!("channel {seq} parse failed: {e} — stopping early");
                            break;
                        }
                    }
                }
                Err(e) => {
                    debug!("channel {seq} read error: {e} — stopping early");
                    break;
                }
            }
        }
        Ok(channels)
    }

    /// Write a single channel.
    pub async fn write_channel(
        &mut self,
        seq: u32,
        rx_hz: u32,
        tx_hz: u32,
        rx_css: u16,
        tx_css: u16,
    ) -> Result<()> {
        let payload =
            protocol::encode_channel_write_request(seq, rx_hz, tx_hz, rx_css, tx_css);
        let frame = self.send_recv(CMD_CHANNEL_WRITE, &payload).await?;
        // Response payload field 1 = result (1 = ok); just log it
        debug!("write channel {seq} response: {} bytes", frame.payload.len());
        Ok(())
    }

    // ── Transport helpers ─────────────────────────────────────────────────────

    async fn send_recv(&mut self, cmd_id: u16, payload: &[u8]) -> Result<protocol::ParsedFrame> {
        let encrypted = self.keys.encrypt(payload);
        let frame = protocol::build_frame(cmd_id, &encrypted);
        self.write_raw(&frame).await?;
        self.wait_frame(RESPONSE_TIMEOUT).await
    }

    async fn write_raw(&self, data: &[u8]) -> Result<()> {
        self.peripheral
            .write(&self.write_char, data, WriteType::WithoutResponse)
            .await
            .map_err(|e| anyhow!("BLE write failed: {e}"))
    }

    async fn wait_frame(&mut self, timeout: Duration) -> Result<protocol::ParsedFrame> {
        let result = tokio::time::timeout(timeout, self.stream.next()).await;
        match result {
            Err(_) => bail!("timeout waiting for BLE notification ({timeout:?})"),
            Ok(None) => bail!("BLE notification stream closed unexpectedly"),
            Ok(Some(notif)) => protocol::parse_frame(&notif.value),
        }
    }

    /// Read all channels and convert directly to a URC profile.
    pub async fn read_profile(&mut self, device_id: &str) -> Result<UrcProfile> {
        let channels = self.read_all_channels().await?;
        Ok(channels_to_urc(device_id, channels))
    }

    /// Gracefully disconnect.
    pub async fn disconnect(self) -> Result<()> {
        self.peripheral
            .disconnect()
            .await
            .map_err(|e| anyhow!("BLE disconnect: {e}"))
    }
}

// ── URC conversion ────────────────────────────────────────────────────────────

/// Convert raw channel data read from a device into a URC-v1 profile.
pub fn channels_to_urc(device_id: &str, channels: Vec<RawChannelInfo>) -> UrcProfile {
    let mut profile = UrcProfile::new(format!("Xiaomi WT2 ({device_id})"));
    profile.devices.push(DeviceBinding {
        model: "xiaomi_walkie_talkie_2".into(),
        transport: "ble".into(),
        adapter: "xiaomi_ble".into(),
    });
    for ch in channels {
        let (rx_type, rx_code) = protocol::decode_css(ch.rx_css);
        let (tx_type, tx_code) = protocol::decode_css(ch.tx_css);
        // Use rx tone for both rx and tx (most simplex configs share the same tone).
        let tone_type = match (rx_code, rx_type) {
            (0, _) => ToneType::None,
            (_, 0) => ToneType::Ctcss,
            _ => ToneType::Dcs,
        };
        let _ = (tx_type, tx_code); // tx tone stored separately if needed in future
        profile.channels.push(Channel {
            id: format!("xiaomi-ch{:02}", ch.seq),
            name: format!("CH{:02}", ch.seq),
            mode: ChannelMode::AnalogFm,
            rx_freq_mhz: protocol::hz_to_mhz(ch.rx_hz),
            tx_freq_mhz: protocol::hz_to_mhz(ch.tx_hz),
            power: PowerLevel::Low,
            bandwidth: Bandwidth::Narrow,
            tags: vec!["xiaomi".into()],
            analog: Some(AnalogFields {
                tone_type,
                tone_value_hz: if rx_code != 0 { Some(rx_code as f64) } else { None },
                scan: true,
            }),
            dmr: None,
        });
    }
    profile
}

/// Convert a URC-v1 profile into per-channel write parameters for the device.
/// Channels are indexed 1-based (slot 1 = first URC channel).
pub fn urc_to_write_params(profile: &UrcProfile) -> Vec<(u32, u32, u32, u16, u16)> {
    profile
        .channels
        .iter()
        .enumerate()
        .map(|(i, ch)| {
            let (tone_type_u8, code_u8) = match ch.analog.as_ref() {
                Some(a) => {
                    let tt: u8 = match a.tone_type {
                        ToneType::Ctcss => 0,
                        ToneType::Dcs => 1,
                        ToneType::None => 0,
                    };
                    (tt, a.tone_value_hz.unwrap_or(0.0) as u8)
                }
                None => (0, 0),
            };
            let css = protocol::encode_css(tone_type_u8, code_u8);
            (
                (i + 1) as u32,
                protocol::mhz_to_hz(ch.rx_freq_mhz),
                protocol::mhz_to_hz(ch.tx_freq_mhz),
                css,
                css,
            )
        })
        .collect()
}
