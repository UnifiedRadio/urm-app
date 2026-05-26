// Xiaomi/极蜂 "Lite" BLE protocol — reverse-engineered from:
//   Mi-Walkie-Talkie-by-Darkhorse/Mi-Walkie-Talkie-Plus (branch 2.13.7-plus)
//   original-java/com/ifengyu/blelib/d/e.java  (Packet.java — frame format)
//   original-java/com/ifengyu/blelib/d/f.java  (PacketUtil.java)
//   original-java/com/ifengyu/blelib/d/b.java  (CRC16.java)
//   original-java/com/ifengyu/intercom/lite/e/f.java (LiteBleClient.java — cmdIds)
//   original-java/com/ifengyu/intercom/lite/e/h/a.java (CmdHelper.java — payload builders)
//
// ## Frame format
//
//   [0x00]: 0xFE  (prefix)
//   [0x01]: 0x01  (version)
//   [0x02..0x03]: total_length = payload_len + 10  (big-endian u16)
//   [0x04..0x05]: cmdId  (big-endian u16)
//   [0x06..0x07]: seq    (big-endian u16, 1..=65535 incrementing)
//   [0x08..]:     protobuf payload
//   [last 2]:     CRC16  (big-endian, covers all preceding bytes)
//
// ## Command IDs (LiteProtos)
//   20001 – ConnectRequest   (→ device, unencrypted — initial handshake)
//   20002 – ConnectResponse  (← device, unencrypted)
//   20003 – DevRequest       (→ device, get device params)
//   20005 – DevRequest       (→ device, set device param)
//   20007 – ChannelInfoRequest w/ seq only   (→ device, read one channel)
//   20012 – ChannelInfoRequest w/ full data  (→ device, write one channel)
//   20014 – OtaRequest start
//   20018 – BatchBrodCfgRequest cancel
//   20020 – BatchBrodCfgRequest start
//   20022 – BatchBrodFskResponse (← device)
//   20023 – OTA firmware data   (→ device, encrypted with ECDH session key)
//   20024 – LoginResponse       (← device, encrypted — contains AES key + IV + token)
//
// ## Authentication flow (abbreviated)
//   1. Connect BLE, request MTU 500, enable notifications on NOTIFY_CHAR.
//   2. Send cmdId=20001 (ConnectRequest):
//        { appType: FY_APP(0), version: 1, deviceId: <saved_id>, publicKey: <ECDH pubkey> }
//      unencrypted.
//   3. Receive cmdId=20002 or 20024:
//      - 20002/ALLOW  → no key exchange; proceed to cmdId=20003.
//      - 20024/ALLOW  → AES key+IV embedded; store for future commands; proceed to 20003.
//      - 20002/WAIT   → show "confirm on device" UI; wait up to 20 s.
//      - REFUSE / LOWPOWER / other → disconnect.
//   4. Send cmdId=20003 (DevRequest version=1) to read device params.
//   5. Read/write channels with cmdId=20007/20012.
//
// ## Protobuf encoding (hand-coded — no prost dependency required)
//
//   LiteProtos.ChannelInfo:
//     field 1 (seq):    varint  — channel number, 1-based
//     field 2 (rxFreq): varint  — receive frequency in Hz (e.g. 430_000_000)
//     field 3 (txFreq): varint  — transmit frequency in Hz
//     field 4 (rxCss):  varint  — receive tone: (tone_type+1)<<8 | code; 0 = none
//     field 5 (txCss):  varint  — transmit tone: same encoding
//
//   LiteProtos.ChannelInfoRequest (wraps ChannelInfo):
//     field 1 (version):     varint 1
//     field 2 (channelInfo): length-delimited ChannelInfo bytes
//
//   LiteProtos.ConnectRequest (cmdId 20001):
//     field 1 (version):    varint 1
//     field 2 (appType):    varint 0 (FY_APP)
//     field 3 (deviceId):   varint (omit if unknown)
//     field 5 (publicKey):  bytes  (omit if no ECDH)
//
//   LiteProtos.LoginRequest (cmdId 20001 re-login with token):
//     field 1 (version): varint 1
//     field 2 (token):   bytes

use anyhow::{bail, Result};
use std::sync::atomic::{AtomicU16, Ordering};

// ── Command IDs ──────────────────────────────────────────────────────────────

pub const CMD_CONNECT_REQUEST: u16 = 20001;
pub const CMD_CONNECT_RESPONSE: u16 = 20002;
pub const CMD_DEV_REQUEST: u16 = 20003;
pub const CMD_CHANNEL_READ: u16 = 20007;
pub const CMD_CHANNEL_WRITE: u16 = 20012;

// ── Sequence counter ─────────────────────────────────────────────────────────

static SEQ: AtomicU16 = AtomicU16::new(1);

fn next_seq() -> u16 {
    loop {
        let prev = SEQ.load(Ordering::Relaxed);
        let next = if prev >= 65535 { 1 } else { prev + 1 };
        if SEQ.compare_exchange(prev, next, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            return next;
        }
    }
}

// ── CRC16 ─────────────────────────────────────────────────────────────────────
// Ported directly from com/ifengyu/blelib/d/b.java (CRC16.java).

#[rustfmt::skip]
const CRC_TABLE: [u16; 256] = [
    0,     4489,  8978,  12955, 17956, 22445, 25910, 29887,
    35912, 40385, 44890, 48851, 51820, 56293, 59774, 63735,
    4225,  264,   13203, 8730,  22181, 18220, 30135, 25662,
    40137, 36160, 49115, 44626, 56045, 52068, 63999, 59510,
    8450,  12427, 528,   5017,  26406, 30383, 17460, 21949,
    44362, 48323, 36440, 40913, 60270, 64231, 51324, 55797,
    12675, 8202,  4753,  792,   30631, 26158, 21685, 17724,
    48587, 44098, 40665, 36688, 64495, 60006, 55549, 51572,
    16900, 21389, 24854, 28831, 1056,  5545,  10034, 14011,
    52812, 57285, 60766, 64727, 34920, 39393, 43898, 47859,
    21125, 17164, 29079, 24606, 5281,  1320,  14259, 9786,
    57037, 53060, 64991, 60502, 39145, 35168, 48123, 43634,
    25350, 29327, 16404, 20893, 9506,  13483, 1584,  6073,
    61262, 65223, 52316, 56789, 43370, 47331, 35448, 39921,
    29575, 25102, 20629, 16668, 13731, 9258,  5809,  1848,
    65487, 60998, 56541, 52564, 47595, 43106, 39673, 35696,
    33800, 38273, 42778, 46739, 49708, 54181, 57662, 61623,
    2112,  6601,  11090, 15067, 20068, 24557, 28022, 31999,
    38025, 34048, 47003, 42514, 53933, 49956, 61887, 57398,
    6337,  2376,  15315, 10842, 24293, 20332, 32247, 27774,
    42250, 46211, 34328, 38801, 58158, 62119, 49212, 53685,
    10562, 14539, 2640,  7129,  28518, 32495, 19572, 24061,
    46475, 41986, 38553, 34576, 62383, 57894, 53437, 49460,
    14787, 10314, 6865,  2904,  32743, 28270, 23797, 19836,
    50700, 55173, 58654, 62615, 32808, 37281, 41786, 45747,
    19012, 23501, 26966, 30943, 3168,  7657,  12146, 16123,
    54925, 50948, 62879, 58390, 37033, 33056, 46011, 41522,
    23237, 19276, 31191, 26718, 7393,  3432,  16371, 11898,
    59150, 63111, 50204, 54677, 41258, 45219, 33336, 37809,
    27462, 31439, 18516, 23005, 11618, 15595, 3696,  8185,
    63375, 58886, 54429, 50452, 45483, 40994, 37561, 33584,
    31687, 27214, 22741, 18780, 15843, 11370, 7921,  3960,
];

fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc = CRC_TABLE[((crc ^ b as u16) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc
}

// ── Protobuf helpers ──────────────────────────────────────────────────────────

fn pb_varint(buf: &mut Vec<u8>, field: u8, value: u64) {
    if value == 0 {
        return; // proto3 default — omit zero values
    }
    buf.push((field << 3) | 0); // wire type 0 = varint
    let mut v = value;
    while v > 0x7F {
        buf.push((v as u8 & 0x7F) | 0x80);
        v >>= 7;
    }
    buf.push(v as u8);
}

fn pb_bytes(buf: &mut Vec<u8>, field: u8, data: &[u8]) {
    if data.is_empty() {
        return;
    }
    buf.push((field << 3) | 2); // wire type 2 = length-delimited
    let mut len = data.len() as u64;
    while len > 0x7F {
        buf.push((len as u8 & 0x7F) | 0x80);
        len >>= 7;
    }
    buf.push(len as u8);
    buf.extend_from_slice(data);
}

fn pb_message(buf: &mut Vec<u8>, field: u8, inner: &[u8]) {
    pb_bytes(buf, field, inner);
}

// ── Channel info protobuf ─────────────────────────────────────────────────────

/// Encode a `LiteProtos.ChannelInfo` message.
/// - `seq`: 1-based channel number
/// - `rx_hz` / `tx_hz`: frequencies in Hz (e.g. 430_000_000 for 430 MHz)
/// - `rx_css` / `tx_css`: `(tone_type + 1) << 8 | tone_code`; 0 = no tone
pub fn encode_channel_info(seq: u32, rx_hz: u32, tx_hz: u32, rx_css: u16, tx_css: u16) -> Vec<u8> {
    let mut buf = Vec::with_capacity(32);
    pb_varint(&mut buf, 1, seq as u64);
    pb_varint(&mut buf, 2, rx_hz as u64);
    pb_varint(&mut buf, 3, tx_hz as u64);
    pb_varint(&mut buf, 4, rx_css as u64);
    pb_varint(&mut buf, 5, tx_css as u64);
    buf
}

/// Encode a `LiteProtos.ChannelInfoRequest` for a channel **read** (cmdId 20007).
/// Only `seq` matters; rx/tx/css fields are zero.
pub fn encode_channel_read_request(seq: u32) -> Vec<u8> {
    let channel_info = encode_channel_info(seq, 0, 0, 0, 0);
    let mut buf = Vec::with_capacity(16);
    pb_varint(&mut buf, 1, 1); // version = 1
    pb_message(&mut buf, 2, &channel_info);
    buf
}

/// Encode a `LiteProtos.ChannelInfoRequest` for a channel **write** (cmdId 20012).
pub fn encode_channel_write_request(
    seq: u32,
    rx_hz: u32,
    tx_hz: u32,
    rx_css: u16,
    tx_css: u16,
) -> Vec<u8> {
    let channel_info = encode_channel_info(seq, rx_hz, tx_hz, rx_css, tx_css);
    let mut buf = Vec::with_capacity(48);
    pb_varint(&mut buf, 1, 1); // version = 1
    pb_message(&mut buf, 2, &channel_info);
    buf
}

/// Encode a minimal `LiteProtos.ConnectRequest` (cmdId 20001, unencrypted).
/// Pass `device_id = 0` and empty `public_key` for a first-time or anonymous connect.
pub fn encode_connect_request(device_id: u32, public_key: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(32);
    pb_varint(&mut buf, 1, 1); // version = 1
    pb_varint(&mut buf, 2, 0); // appType = FY_APP (0)
    if device_id != 0 {
        pb_varint(&mut buf, 3, device_id as u64);
    }
    if !public_key.is_empty() {
        pb_bytes(&mut buf, 5, public_key);
    }
    buf
}

// ── Frame builder/parser ──────────────────────────────────────────────────────

/// Build a complete BLE frame from a `cmdId` and a protobuf `payload`.
pub fn build_frame(cmd_id: u16, payload: &[u8]) -> Vec<u8> {
    let total_len = (payload.len() + 10) as u16;
    let seq = next_seq();

    let mut frame = Vec::with_capacity(payload.len() + 10);
    frame.push(0xFE); // prefix
    frame.push(0x01); // version
    frame.extend_from_slice(&total_len.to_be_bytes());
    frame.extend_from_slice(&cmd_id.to_be_bytes());
    frame.extend_from_slice(&seq.to_be_bytes());
    frame.extend_from_slice(payload);

    let crc = crc16(&frame);
    frame.extend_from_slice(&crc.to_be_bytes());
    frame
}

pub struct ParsedFrame {
    pub cmd_id: u16,
    pub seq: u16,
    pub payload: Vec<u8>,
}

/// Parse and CRC-validate an incoming BLE notification frame.
pub fn parse_frame(data: &[u8]) -> Result<ParsedFrame> {
    if data.len() < 10 {
        bail!("frame too short: {} bytes", data.len());
    }
    if data[0] != 0xFE {
        bail!("bad prefix: 0x{:02X}", data[0]);
    }
    let total_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    if data.len() < total_len {
        bail!("truncated frame: have {} bytes, need {}", data.len(), total_len);
    }
    let body = &data[..total_len - 2];
    let received_crc = u16::from_be_bytes([data[total_len - 2], data[total_len - 1]]);
    let computed_crc = crc16(body);
    if received_crc != computed_crc {
        bail!("CRC mismatch: received 0x{:04X}, computed 0x{:04X}", received_crc, computed_crc);
    }
    let cmd_id = u16::from_be_bytes([data[4], data[5]]);
    let seq = u16::from_be_bytes([data[6], data[7]]);
    let payload = data[8..total_len - 2].to_vec();
    Ok(ParsedFrame { cmd_id, seq, payload })
}

// ── Frequency helpers ─────────────────────────────────────────────────────────

/// Convert MHz (URC) to the integer Hz used in the Lite protocol.
pub fn mhz_to_hz(mhz: f64) -> u32 {
    (mhz * 1_000_000.0).round() as u32
}

/// Convert Hz (Lite protocol) back to MHz for URC.
pub fn hz_to_mhz(hz: u32) -> f64 {
    hz as f64 / 1_000_000.0
}

/// Encode a CTCSS/DCS tone into the Lite CSS field.
/// - `tone_type`: 0 = CTCSS, 1 = DCS (matches the `receiveToneType` convention)
/// - `tone_code`: numeric code (0 means "none" regardless of type)
pub fn encode_css(tone_type: u8, tone_code: u8) -> u16 {
    if tone_code == 0 {
        0
    } else {
        ((tone_type as u16 + 1) << 8) | tone_code as u16
    }
}

/// Decode a Lite CSS field into (tone_type 0-indexed, tone_code); (0, 0) = none.
pub fn decode_css(css: u16) -> (u8, u8) {
    if css == 0 {
        return (0, 0);
    }
    let tone_type = ((css >> 8) & 0xFF) as u8 - 1;
    let tone_code = (css & 0xFF) as u8;
    (tone_type, tone_code)
}

// ── Protobuf response reader ──────────────────────────────────────────────────

#[derive(Debug)]
pub enum PbValue {
    Varint(u64),
    Bytes(Vec<u8>),
}

/// Read a single protobuf (field_number, wire_type, value) at `pos`.
/// Returns `(field_number, value, next_pos)`.
pub fn pb_read_field(data: &[u8], pos: usize) -> Result<(u32, PbValue, usize)> {
    let (tag, pos) = read_varint(data, pos)?;
    let field = (tag >> 3) as u32;
    let wire = (tag & 0x07) as u8;
    match wire {
        0 => {
            let (v, next) = read_varint(data, pos)?;
            Ok((field, PbValue::Varint(v), next))
        }
        1 => {
            // 64-bit fixed — skip
            if pos + 8 > data.len() {
                bail!("fixed64 field overruns buffer");
            }
            Ok((field, PbValue::Bytes(data[pos..pos + 8].to_vec()), pos + 8))
        }
        2 => {
            let (len, pos) = read_varint(data, pos)?;
            let end = pos + len as usize;
            if end > data.len() {
                bail!("length-delimited field overruns buffer");
            }
            Ok((field, PbValue::Bytes(data[pos..end].to_vec()), end))
        }
        5 => {
            // 32-bit fixed — skip
            if pos + 4 > data.len() {
                bail!("fixed32 field overruns buffer");
            }
            Ok((field, PbValue::Bytes(data[pos..pos + 4].to_vec()), pos + 4))
        }
        wt => bail!("unsupported wire type {wt} at field {field}"),
    }
}

fn read_varint(data: &[u8], mut pos: usize) -> Result<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    loop {
        if pos >= data.len() {
            bail!("truncated varint");
        }
        let b = data[pos];
        pos += 1;
        result |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            return Ok((result, pos));
        }
        shift += 7;
        if shift >= 64 {
            bail!("varint overflow");
        }
    }
}

// ── ConnectResponse / LoginResponse ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectResult {
    Allow,
    Wait,
    Refuse,
    LowPower,
    Unknown(u64),
}

impl From<u64> for ConnectResult {
    fn from(v: u64) -> Self {
        match v {
            1 => Self::Allow,
            2 => Self::Wait,
            3 => Self::Refuse,
            4 => Self::LowPower,
            n => Self::Unknown(n),
        }
    }
}

pub struct ConnectResponse {
    pub result: ConnectResult,
    /// Device ECDH public key (field 2 in LoginResponse, SEC1 uncompressed bytes).
    pub device_public_key: Option<Vec<u8>>,
    /// AES-encrypted session data (field 3): aes_key(16) || aes_iv(16) || token.
    pub encrypted_session_data: Option<Vec<u8>>,
}

/// Parse a ConnectResponse (cmdId 20002) or LoginResponse (cmdId 20024) payload.
pub fn parse_connect_response(payload: &[u8]) -> Result<ConnectResponse> {
    let mut result = ConnectResult::Unknown(0);
    let mut device_public_key = None;
    let mut encrypted_session_data = None;
    let mut pos = 0;
    while pos < payload.len() {
        let (field, value, next) = pb_read_field(payload, pos)?;
        pos = next;
        match (field, value) {
            (1, PbValue::Varint(v)) => result = ConnectResult::from(v),
            (2, PbValue::Bytes(b)) => device_public_key = Some(b),
            (3, PbValue::Bytes(b)) => encrypted_session_data = Some(b),
            _ => {}
        }
    }
    Ok(ConnectResponse { result, device_public_key, encrypted_session_data })
}

// ── Channel response decoder ──────────────────────────────────────────────────

pub struct RawChannelInfo {
    pub seq: u32,
    pub rx_hz: u32,
    pub tx_hz: u32,
    pub rx_css: u16,
    pub tx_css: u16,
}

/// Decode a `LiteProtos.ChannelInfo` message.
pub fn decode_channel_info(data: &[u8]) -> Result<RawChannelInfo> {
    let mut seq = 0u64;
    let mut rx_hz = 0u64;
    let mut tx_hz = 0u64;
    let mut rx_css = 0u64;
    let mut tx_css = 0u64;
    let mut pos = 0;
    while pos < data.len() {
        let (field, value, next) = pb_read_field(data, pos)?;
        pos = next;
        if let PbValue::Varint(v) = value {
            match field {
                1 => seq = v,
                2 => rx_hz = v,
                3 => tx_hz = v,
                4 => rx_css = v,
                5 => tx_css = v,
                _ => {}
            }
        }
    }
    Ok(RawChannelInfo {
        seq: seq as u32,
        rx_hz: rx_hz as u32,
        tx_hz: tx_hz as u32,
        rx_css: rx_css as u16,
        tx_css: tx_css as u16,
    })
}

/// Decode a `LiteProtos.ChannelInfoRequest` wrapper (field 2 = ChannelInfo).
pub fn decode_channel_info_request(data: &[u8]) -> Result<RawChannelInfo> {
    let mut pos = 0;
    while pos < data.len() {
        let (field, value, next) = pb_read_field(data, pos)?;
        pos = next;
        if field == 2 {
            if let PbValue::Bytes(inner) = value {
                return decode_channel_info(&inner);
            }
        }
    }
    bail!("no channelInfo field (field 2) found in ChannelInfoRequest response")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_frame() {
        let payload = encode_channel_read_request(1);
        let frame = build_frame(CMD_CHANNEL_READ, &payload);
        let parsed = parse_frame(&frame).unwrap();
        assert_eq!(parsed.cmd_id, CMD_CHANNEL_READ);
        assert_eq!(parsed.payload, payload);
    }

    #[test]
    fn crc_known_empty() {
        // CRC16 of empty = 0xFFFF; big-endian bytes [0xFF, 0xFF]
        assert_eq!(crc16(&[]), 0xFFFF);
    }

    #[test]
    fn freq_roundtrip() {
        let hz = mhz_to_hz(430.0125);
        assert_eq!(hz_to_mhz(hz), 430.0125);
    }

    #[test]
    fn css_roundtrip() {
        let css = encode_css(0, 67); // CTCSS code 67
        let (t, c) = decode_css(css);
        assert_eq!((t, c), (0, 67));

        assert_eq!(encode_css(0, 0), 0); // no tone
        assert_eq!(decode_css(0), (0, 0));
    }
}
