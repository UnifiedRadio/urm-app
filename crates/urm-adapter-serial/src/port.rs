use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PortInfo {
    pub name: String,
    pub description: Option<String>,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
}

/// List all available serial ports on the system.
pub fn list_ports() -> anyhow::Result<Vec<PortInfo>> {
    let ports = serialport::available_ports()?;
    Ok(ports.into_iter().map(|p| {
        let (vid, pid, desc) = match &p.port_type {
            serialport::SerialPortType::UsbPort(info) => (
                Some(info.vid),
                Some(info.pid),
                info.product.clone(),
            ),
            _ => (None, None, None),
        };
        PortInfo { name: p.port_name, description: desc, vid, pid }
    }).collect())
}
