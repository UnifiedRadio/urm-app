use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command;

static CACHED_QDMR_BIN: OnceLock<Option<String>> = OnceLock::new();

#[derive(Default)]
pub struct QdmrEnv;

impl QdmrEnv {
    pub fn detect() -> Self {
        Self
    }

    pub async fn run(&self, args: &[&str]) -> anyhow::Result<String> {
        let bin = resolve_qdmr_cli()
            .await
            .ok_or_else(|| anyhow::anyhow!("qdmr not found. Install qDMR from https://github.com/hmatuschek/qdmr"))?;

        let output = Command::new(&bin)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            anyhow::bail!("qdmr exited with error: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn run_to_file(&self, args: &[&str]) -> anyhow::Result<()> {
        let bin = resolve_qdmr_cli()
            .await
            .ok_or_else(|| anyhow::anyhow!("qdmr not found. Install qDMR from https://github.com/hmatuschek/qdmr"))?;

        let status = Command::new(&bin)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("qdmr exited with non-zero status");
        }
        Ok(())
    }
}

async fn resolve_qdmr_cli() -> Option<String> {
    if let Some(cached) = CACHED_QDMR_BIN.get() {
        return cached.clone();
    }
    let result = find_qdmr().await;
    let _ = CACHED_QDMR_BIN.set(result.clone());
    result
}

async fn find_qdmr() -> Option<String> {
    for candidate in &["qdmr"] {
        if Command::new(candidate).arg("--version").output().await.is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}
