use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PortProbeResult {
    pub port: String,
    pub accessible: bool,
    pub driver: DriverFingerprint,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DriverFingerprint {
    Genuine,
    CounterfeitProlific,
    CounterfeitFtdi,
    Ch340,
    Unknown,
}

// Known counterfeit USB VID/PID combinations
// Prolific 0x067B/0x2303 clones are the most common in cheap UV-5R cables
const COUNTERFEIT_PROLIFIC: &[(u16, u16)] = &[(0x067B, 0x2303)];

pub fn probe_port(port_name: &str, vid: Option<u16>, pid: Option<u16>) -> PortProbeResult {
    let driver = fingerprint_driver(vid, pid);
    let (accessible, error) = check_port_accessible(port_name);

    PortProbeResult { port: port_name.to_string(), accessible, driver, error }
}

fn fingerprint_driver(vid: Option<u16>, pid: Option<u16>) -> DriverFingerprint {
    match (vid, pid) {
        (Some(v), Some(p)) if COUNTERFEIT_PROLIFIC.contains(&(v, p)) => {
            DriverFingerprint::CounterfeitProlific
        }
        (Some(0x1A86), _) => DriverFingerprint::Ch340,
        (Some(0x0403), _) => DriverFingerprint::Genuine, // genuine FTDI
        (Some(_), Some(_)) => DriverFingerprint::Genuine,
        _ => DriverFingerprint::Unknown,
    }
}

/// Actually attempt to open the port at the lowest baud rate to detect
/// permission errors. `fs::metadata` on macOS returns Ok for tty devices
/// even when the process lacks access rights.
fn check_port_accessible(port_name: &str) -> (bool, Option<String>) {
    match serialport::new(port_name, 9600)
        .timeout(std::time::Duration::from_millis(100))
        .open()
    {
        Ok(_) => (true, None),
        Err(e) => {
            let msg = match e.kind() {
                serialport::ErrorKind::NoDevice => {
                    "Port not found — check cable connection".into()
                }
                serialport::ErrorKind::Io(io_kind) if io_kind == std::io::ErrorKind::PermissionDenied => {
                    "Permission denied — check serial port access rights".into()
                }
                _ => e.to_string(),
            };
            (false, Some(msg))
        }
    }
}
