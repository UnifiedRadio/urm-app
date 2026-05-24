/// Smart Diagnosis Engine — translates raw hardware errors into human-readable guidance.
/// See design.md §5.3 for the 8-step diagnosis pipeline.

#[derive(Debug, serde::Serialize)]
pub struct DiagnosisReport {
    pub step: DiagnosisStep,
    pub status: DiagnosisStatus,
    pub user_message: String,
    pub action_hint: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisStep {
    PortEnumeration,
    DriverFingerprint,
    PermissionCheck,
    DeviceHandshake,
    DryRun,
    Backup,
    Write,
    PostWriteVerify,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosisStatus {
    Ok,
    Warning,
    Error,
    UserActionRequired,
}

/// Translate a raw error string from an adapter into a user-facing DiagnosisReport.
pub fn translate_error(raw: &str, step: DiagnosisStep) -> DiagnosisReport {
    // Pattern match known error signatures
    if raw.contains("Permission denied") || raw.contains("Access denied") {
        return DiagnosisReport {
            step,
            status: DiagnosisStatus::UserActionRequired,
            user_message: "Serial port access denied. The application needs permission to access USB devices.".into(),
            action_hint: Some("On macOS: approve in System Settings → Privacy → USB. On Linux: run `sudo usermod -aG dialout $USER` then log out and back in.".into()),
        };
    }

    if raw.contains("No such file") || raw.contains("not found") {
        return DiagnosisReport {
            step,
            status: DiagnosisStatus::Error,
            user_message: "No programming cable detected. Please check the USB cable is connected.".into(),
            action_hint: Some("Make sure the cable is fully inserted and the radio is powered on.".into()),
        };
    }

    if raw.contains("timeout") || raw.contains("no response") {
        return DiagnosisReport {
            step,
            status: DiagnosisStatus::Error,
            user_message: "Radio is not responding. Is it powered on with the volume knob turned up?".into(),
            action_hint: None,
        };
    }

    DiagnosisReport {
        step,
        status: DiagnosisStatus::Error,
        user_message: "An unexpected error occurred. Check the log for details.".into(),
        action_hint: None,
    }
}
