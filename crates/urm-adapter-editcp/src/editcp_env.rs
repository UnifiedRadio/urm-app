use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command;

static CACHED_EDITCP_BIN: OnceLock<Option<String>> = OnceLock::new();

#[derive(Default)]
pub struct EditcpEnv;

impl EditcpEnv {
    pub fn detect() -> Self {
        Self
    }

    pub async fn run(&self, args: &[&str]) -> anyhow::Result<String> {
        let bin = resolve_editcp()
            .await
            .ok_or_else(|| anyhow::anyhow!("editcp not found. Install from https://github.com/DaleFarnsworth-DMR/editcp"))?;

        let output = Command::new(&bin)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            anyhow::bail!("editcp error: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn run_silent(&self, args: &[&str]) -> anyhow::Result<()> {
        let bin = resolve_editcp()
            .await
            .ok_or_else(|| anyhow::anyhow!("editcp not found. Install from https://github.com/DaleFarnsworth-DMR/editcp"))?;

        let status = Command::new(&bin)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("editcp exited with non-zero status");
        }
        Ok(())
    }
}

async fn resolve_editcp() -> Option<String> {
    if let Some(cached) = CACHED_EDITCP_BIN.get() {
        return cached.clone();
    }
    let result = find_editcp().await;
    let _ = CACHED_EDITCP_BIN.set(result.clone());
    result
}

async fn find_editcp() -> Option<String> {
    for candidate in &["editcp"] {
        if Command::new(candidate).arg("--version").output().await.is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}
