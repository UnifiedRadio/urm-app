// Phase 3 — Xiaomi Walkie Talkie 2 / 极蜂 A108Plus BLE adapter
// Protocol reverse-engineered from Mi-Walkie-Talkie-by-Darkhorse/Mi-Walkie-Talkie-Plus.
// Hardware validation pending real-device testing.

mod crypto;
mod gatt;
mod protocol;

pub mod client;
mod scanner;

pub use client::{channels_to_urc, scan_xiaomi_devices, urc_to_write_params, BleSession, ScannedDevice};
pub use protocol::RawChannelInfo;
pub use scanner::XiaomiAdapter;
