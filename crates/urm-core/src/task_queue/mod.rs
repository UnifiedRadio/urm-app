use tokio::sync::Mutex;
use std::sync::Arc;
use crate::adapter::{RadioAdapter, Transport, AdapterError, WriteReport};
use crate::backup::{BackupManager, BackupMeta};
use crate::schema::UrcProfile;

/// Serializes all hardware operations — prevents concurrent access to the same device.
/// Enforces the read-backup-write-verify contract defined in design.md §5.3.
pub struct TaskQueue {
    lock: Arc<Mutex<()>>,
    backup_manager: Arc<BackupManager>,
}

impl TaskQueue {
    pub fn new(backup_manager: Arc<BackupManager>) -> Self {
        Self {
            lock: Arc::new(Mutex::new(())),
            backup_manager,
        }
    }

    pub fn backup_manager(&self) -> &BackupManager {
        &self.backup_manager
    }

    /// Read configuration from device. Serialized to prevent overlap with writes.
    pub async fn read(
        &self,
        adapter: &dyn RadioAdapter,
        transport: &Transport,
    ) -> Result<UrcProfile, AdapterError> {
        let _guard = self.lock.lock().await;
        adapter.read(transport).await
    }

    /// Write with mandatory dry-run → backup → write pipeline.
    /// Aborts if backup fails unless `force_without_backup` is explicitly set.
    pub async fn write(
        &self,
        adapter: &dyn RadioAdapter,
        transport: &Transport,
        profile: &UrcProfile,
        force_without_backup: bool,
    ) -> Result<WriteReport, AdapterError> {
        let _guard = self.lock.lock().await;

        // Step 1: dry-run validation
        let dry_run = adapter.dry_run(transport, profile).await?;
        if !dry_run.valid {
            return Err(AdapterError::WriteAborted(
                format!("{} validation error(s) — write blocked", dry_run.errors.len()),
            ));
        }

        // Step 2: mandatory backup before any destructive write
        match adapter.backup(transport).await {
            Ok(image) => {
                self.backup_manager
                    .save(&image, &profile.id)
                    .await
                    .map_err(|e| AdapterError::BackupFailed(e.to_string()))?;
            }
            Err(e) if !force_without_backup => {
                return Err(AdapterError::BackupFailed(format!(
                    "backup failed, write aborted: {e}"
                )));
            }
            Err(e) => {
                tracing::warn!("backup failed but force_without_backup=true: {e}");
            }
        }

        // Step 3: write
        adapter.write(transport, profile).await
    }

    /// Explicit backup without write (for manual backup command).
    pub async fn backup(
        &self,
        adapter: &dyn RadioAdapter,
        transport: &Transport,
        profile_id: &str,
    ) -> Result<BackupMeta, AdapterError> {
        let _guard = self.lock.lock().await;
        let image = adapter.backup(transport).await?;
        self.backup_manager
            .save(&image, profile_id)
            .await
            .map_err(|e| AdapterError::BackupFailed(e.to_string()))
    }
}
