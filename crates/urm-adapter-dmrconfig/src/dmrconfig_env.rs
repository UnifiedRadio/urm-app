use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command;

static CACHED_BIN: OnceLock<Option<String>> = OnceLock::new();

#[derive(Default)]
pub struct DmrconfigEnv;

impl DmrconfigEnv {
    pub fn detect() -> Self {
        Self
    }

    pub async fn run(&self, args: &[&str]) -> anyhow::Result<String> {
        let bin = resolve().await.ok_or_else(|| {
            anyhow::anyhow!("dmrconfig not found. Install from https://github.com/OpenRTX/dmrconfig")
        })?;

        let output = Command::new(&bin)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            anyhow::bail!("dmrconfig error: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn run_silent(&self, args: &[&str]) -> anyhow::Result<()> {
        let bin = resolve().await.ok_or_else(|| {
            anyhow::anyhow!("dmrconfig not found. Install from https://github.com/OpenRTX/dmrconfig")
        })?;

        let status = Command::new(&bin)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("dmrconfig exited with non-zero status");
        }
        Ok(())
    }
}

async fn resolve() -> Option<String> {
    if let Some(cached) = CACHED_BIN.get() {
        return cached.clone();
    }
    let result = find_bin().await;
    let _ = CACHED_BIN.set(result.clone());
    result
}

async fn find_bin() -> Option<String> {
    for candidate in &["dmrconfig"] {
        if Command::new(candidate).arg("-v").output().await.is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}
