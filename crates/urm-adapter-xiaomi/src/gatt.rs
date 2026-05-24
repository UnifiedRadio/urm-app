// GATT UUIDs reverse-engineered from Mi-Walkie-Talkie-by-Darkhorse/Mi-Walkie-Talkie-Plus
// Source: original-java/com/ifengyu/intercom/lite/e/f.java (LiteBleClient.java)
//
// The "Lite" protocol (LiteBleClient) serves both:
//   - 小米对讲机 Lite (Xiaomi Walkie Talkie Lite)
//   - 极蜂 A108Plus (via OTA channel copy compatibility)
//
// The older Mi WT2 GATT (BtleCentralService) also advertises:
//   service FEE7, chars AEC7/AEC8/FEAB — but routing logic is in the legacy stack.
// For URM Phase 3 we target the Lite protocol which uses FDAB/AEC7/AEC8.

use uuid::Uuid;

/// Primary BLE service — `0000FDAB-0000-1000-8000-00805f9b34fb`
pub fn service_uuid() -> Uuid {
    Uuid::from_bytes([0x00, 0x00, 0xFD, 0xAB, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34, 0xfb])
}

/// Write-without-response characteristic — `0000AEC7-0000-1000-8000-00805f9b34fb`
pub fn write_char_uuid() -> Uuid {
    Uuid::from_bytes([0x00, 0x00, 0xAE, 0xC7, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34, 0xfb])
}

/// Notify characteristic — `0000AEC8-0000-1000-8000-00805f9b34fb`
pub fn notify_char_uuid() -> Uuid {
    Uuid::from_bytes([0x00, 0x00, 0xAE, 0xC8, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b, 0x34, 0xfb])
}

/// OTA firmware-upgrade service — `00010203-0405-0607-0809-0a0b0c0d1912`
pub fn ota_service_uuid() -> Uuid {
    Uuid::from_bytes([0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x19, 0x12])
}

/// OTA write characteristic — `00010203-0405-0607-0809-0a0b0c0d2b12`
pub fn ota_write_char_uuid() -> Uuid {
    Uuid::from_bytes([0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x2b, 0x12])
}

pub struct GattMap {
    pub service_uuid: Uuid,
    pub channel_write_char: Uuid,
    pub channel_notify_char: Uuid,
}

impl GattMap {
    pub fn xiaomi_wt2() -> Self {
        GattMap {
            service_uuid: service_uuid(),
            channel_write_char: write_char_uuid(),
            channel_notify_char: notify_char_uuid(),
        }
    }
}
