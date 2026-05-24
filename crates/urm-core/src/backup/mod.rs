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
        let stem = format!("img--{}--{}--{}", profile_id, image.device_model, timestamp);
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
        let stem = format!("snap--{}--{}", profile.id, timestamp);
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

            let meta = if let Some(rest) = stem.strip_prefix("snap--") {
                parse_snap_stem(rest, &stem, &path, &file_meta)
            } else if let Some(rest) = stem.strip_prefix("img--") {
                parse_img_stem(rest, &stem, &path, &file_meta)
            } else if let Some(rest) = stem.strip_prefix("snap_") {
                parse_legacy_snap_stem(rest, &stem, &path, &file_meta)
            } else if let Some(rest) = stem.strip_prefix("img_") {
                parse_legacy_img_stem(rest, &stem, &path, &file_meta)
            } else {
                None
            };

            if let Some(m) = meta {
                metas.push(m);
            }
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

// New format: snap--{profile_id}--{timestamp}
fn parse_snap_stem(rest: &str, stem: &str, path: &std::path::Path, file_meta: &std::fs::Metadata) -> Option<BackupMeta> {
    let (pid, _ts) = rest.split_once("--")?;
    Some(build_meta(stem, pid, "", BackupKind::ProfileSnapshot, path, file_meta))
}

// New format: img--{profile_id}--{device_model}--{timestamp}
fn parse_img_stem(rest: &str, stem: &str, path: &std::path::Path, file_meta: &std::fs::Metadata) -> Option<BackupMeta> {
    let (pid, remainder) = rest.split_once("--")?;
    let model = remainder.split_once("--").map(|(m, _)| m).unwrap_or(remainder);
    Some(build_meta(stem, pid, model, BackupKind::DeviceImage, path, file_meta))
}

// Legacy format: snap_{profile_id}_{timestamp}
fn parse_legacy_snap_stem(rest: &str, stem: &str, path: &std::path::Path, file_meta: &std::fs::Metadata) -> Option<BackupMeta> {
    let (pid, _ts) = rest.split_once('_').unwrap_or((rest, ""));
    Some(build_meta(stem, pid, "", BackupKind::ProfileSnapshot, path, file_meta))
}

// Legacy format: img_{profile_id}_{device_model}_{timestamp}
fn parse_legacy_img_stem(rest: &str, stem: &str, path: &std::path::Path, file_meta: &std::fs::Metadata) -> Option<BackupMeta> {
    let (pid, rest2) = rest.split_once('_').unwrap_or((rest, ""));
    let model = rest2.rsplitn(2, '_').last().unwrap_or("").to_string();
    Some(build_meta(stem, pid, &model, BackupKind::DeviceImage, path, file_meta))
}

fn build_meta(
    stem: &str,
    profile_id: &str,
    device_model: &str,
    kind: BackupKind,
    path: &std::path::Path,
    file_meta: &std::fs::Metadata,
) -> BackupMeta {
    let created_at = file_meta
        .modified()
        .map(chrono::DateTime::<chrono::Utc>::from)
        .unwrap_or_else(|_| chrono::Utc::now());
    BackupMeta {
        id: stem.to_string(),
        profile_id: profile_id.to_string(),
        path: path.to_path_buf(),
        device_model: device_model.to_string(),
        kind,
        created_at,
        size_bytes: file_meta.len() as usize,
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
