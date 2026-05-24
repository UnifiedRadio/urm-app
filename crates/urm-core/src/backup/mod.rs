use std::path::PathBuf;
use crate::adapter::BackupImage;
use crate::schema::UrcProfile;

pub struct BackupManager {
    storage_dir: PathBuf,
}

impl BackupManager {
    pub fn new(storage_dir: PathBuf) -> Self {
        Self { storage_dir }
    }

    async fn ensure_dir(&self) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&self.storage_dir).await?;
        Ok(())
    }

    /// Save a raw device image (binary backup before write).
    pub async fn save(
        &self,
        image: &BackupImage,
        profile_id: &str,
    ) -> anyhow::Result<BackupMeta> {
        self.ensure_dir().await?;
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let stem = format!("img_{}_{}_{}", profile_id, image.device_model, timestamp);
        let filename = format!("{}.{}", stem, image.format);
        let path = self.storage_dir.join(&filename);
        tokio::fs::write(&path, &image.data).await?;
        Ok(BackupMeta {
            id: stem,
            profile_id: profile_id.to_string(),
            path,
            device_model: image.device_model.clone(),
            kind: BackupKind::DeviceImage,
            created_at: chrono::Utc::now(),
            size_bytes: image.data.len(),
        })
    }

    /// Save a UrcProfile JSON snapshot (FR-04 version history).
    pub async fn save_snapshot(&self, profile: &UrcProfile) -> anyhow::Result<BackupMeta> {
        self.ensure_dir().await?;
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
        let stem = format!("snap_{}_{}", profile.id, timestamp);
        let path = self.storage_dir.join(format!("{}.json", stem));
        let json = serde_json::to_string_pretty(profile)?;
        let size = json.len();
        tokio::fs::write(&path, json.as_bytes()).await?;
        Ok(BackupMeta {
            id: stem,
            profile_id: profile.id.clone(),
            path,
            device_model: String::new(),
            kind: BackupKind::ProfileSnapshot,
            created_at: chrono::Utc::now(),
            size_bytes: size,
        })
    }

    /// List all backups, newest first.
    pub async fn list(&self) -> anyhow::Result<Vec<BackupMeta>> {
        self.ensure_dir().await?;
        let mut read_dir = tokio::fs::read_dir(&self.storage_dir).await?;
        let mut metas = Vec::new();

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            let file_meta = match entry.metadata().await {
                Ok(m) if m.is_file() => m,
                _ => continue,
            };

            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };

            let (kind, profile_id, device_model) = if let Some(rest) = stem.strip_prefix("snap_") {
                // snap_{profile_id}_{timestamp}  — profile_id is UUID (no underscores)
                let (pid, _ts) = rest.split_once('_').unwrap_or((rest, ""));
                (BackupKind::ProfileSnapshot, pid.to_string(), String::new())
            } else if let Some(rest) = stem.strip_prefix("img_") {
                // img_{profile_id}_{device_model}_{timestamp}
                // profile_id is UUID (hyphens only), device_model may contain underscores
                let (pid, rest2) = rest.split_once('_').unwrap_or((rest, ""));
                // rest2 = "{device_model}_{timestamp}" — strip timestamp from right
                let model = rest2.rsplitn(2, '_').last().unwrap_or("").to_string();
                (BackupKind::DeviceImage, pid.to_string(), model)
            } else {
                continue;
            };

            let created_at = file_meta
                .modified()
                .map(|t| chrono::DateTime::<chrono::Utc>::from(t))
                .unwrap_or_else(|_| chrono::Utc::now());

            metas.push(BackupMeta {
                id: stem,
                profile_id,
                path,
                device_model,
                kind,
                created_at,
                size_bytes: file_meta.len() as usize,
            });
        }

        metas.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(metas)
    }

    /// Restore a profile snapshot by backup id (filename stem).
    pub async fn restore_snapshot(&self, backup_id: &str) -> anyhow::Result<UrcProfile> {
        let path = self.storage_dir.join(format!("{}.json", backup_id));
        let text = tokio::fs::read_to_string(&path).await?;
        let profile: UrcProfile = serde_json::from_str(&text)?;
        Ok(profile)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupKind {
    DeviceImage,
    ProfileSnapshot,
}

#[derive(Debug, serde::Serialize)]
pub struct BackupMeta {
    pub id: String,
    pub profile_id: String,
    pub path: PathBuf,
    pub device_model: String,
    pub kind: BackupKind,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub size_bytes: usize,
}
