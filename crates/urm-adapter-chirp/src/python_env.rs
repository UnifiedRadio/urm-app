use std::process::Stdio;
use tokio::process::Command;

#[derive(Default)]
pub struct PythonEnv;

impl PythonEnv {
    /// Keep constructor infallible; we detect binaries lazily on each call.
    pub fn detect() -> Self {
        Self
    }

    pub async fn run_chirp(&self, args: &[&str]) -> anyhow::Result<String> {
        let chirp_bin = find_chirp_cli().await;
        let mut cmd = if let Some(chirp) = chirp_bin {
            let mut c = Command::new(chirp);
            c.args(args);
            c
        } else {
            let python_bin = find_python().await?;
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
