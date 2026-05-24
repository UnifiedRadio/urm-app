use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command;

static CACHED_CHIRP_BIN: OnceLock<Option<String>> = OnceLock::new();
static CACHED_PYTHON_BIN: OnceLock<Option<String>> = OnceLock::new();

#[derive(Default)]
pub struct PythonEnv;

impl PythonEnv {
    pub fn detect() -> Self {
        Self
    }

    pub async fn run_chirp(&self, args: &[&str]) -> anyhow::Result<String> {
        let chirp_bin = resolve_chirp_cli().await;
        let mut cmd = if let Some(chirp) = chirp_bin {
            let mut c = Command::new(chirp);
            c.args(args);
            c
        } else {
            let python_bin = resolve_python().await?;
            let mut c = Command::new(&python_bin);
            c.arg("-m").arg("chirp").args(args);
            c
        };

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            anyhow::bail!("chirp exited with error: {stderr}");
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

async fn resolve_python() -> anyhow::Result<String> {
    if let Some(cached) = CACHED_PYTHON_BIN.get() {
        return cached.clone().ok_or_else(|| anyhow::anyhow!("Python not found (cached)"));
    }

    let result = find_python().await;
    let _ = CACHED_PYTHON_BIN.set(result.as_ref().ok().cloned());
    result
}

async fn resolve_chirp_cli() -> Option<String> {
    if let Some(cached) = CACHED_CHIRP_BIN.get() {
        return cached.clone();
    }

    let result = find_chirp_cli().await;
    let _ = CACHED_CHIRP_BIN.set(result.clone());
    result
}

async fn find_python() -> anyhow::Result<String> {
    for candidate in &["python3", "python"] {
        if Command::new(candidate).arg("--version").output().await.is_ok() {
            return Ok(candidate.to_string());
        }
    }
    anyhow::bail!("Python not found. Please install Python 3.8+.")
}

async fn find_chirp_cli() -> Option<String> {
    for candidate in &["chirpw", "chirp"] {
        if Command::new(candidate).arg("--version").output().await.is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}
