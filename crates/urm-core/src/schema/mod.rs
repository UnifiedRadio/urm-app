pub mod channel;
pub mod profile;
pub mod validation;

pub use channel::{Channel, ChannelMode, AnalogFields, DmrFields, PowerLevel, Bandwidth, ToneType};
pub use profile::{UrcProfile, DeviceBinding};
pub use validation::ValidationResult;

pub type DeviceModel = String;
