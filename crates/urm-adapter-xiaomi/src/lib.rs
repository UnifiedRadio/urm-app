// Phase 3 — Xiaomi Walkie Talkie 2 BLE adapter
// GATT feasibility must be confirmed in Phase 0 before this is activated.
// See design.md §5.4 and Q1 in §12.

mod gatt;
mod protocol;
mod scanner;

pub use scanner::XiaomiAdapter;
