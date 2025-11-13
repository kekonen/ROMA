use roma_core::{Result, RomaError};
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use uuid::Uuid;

pub struct CheckpointManager {
    storage_path: PathBuf,
    max_checkpoints: usize,
}

impl CheckpointManager {
    pub fn new(storage_path: PathBuf, max_checkpoints: usize) -> Self {
        Self {
            storage_path,
            max_checkpoints,
        }
    }

    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.storage_path)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to create checkpoint directory: {}", e)))?;
        Ok(())
    }

    pub async fn save_checkpoint(
        &self,
        execution_id: &str,
        dag_data: &Value,
        metadata: Option<&Value>,
    ) -> Result<String> {
        let checkpoint_id = Uuid::new_v4().to_string();
        let checkpoint_path = self
            .storage_path
            .join(format!("checkpoint_{}_{}.json", execution_id, checkpoint_id));

        let checkpoint_data = serde_json::json!({
            "checkpoint_id": checkpoint_id,
            "execution_id": execution_id,
            "timestamp": chrono::Utc::now(),
            "dag_data": dag_data,
            "metadata": metadata,
        });

        let content = serde_json::to_string_pretty(&checkpoint_data)
            .map_err(|e| RomaError::SerializationError(e.to_string()))?;

        fs::write(&checkpoint_path, content)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to write checkpoint: {}", e)))?;

        info!("Checkpoint {} saved for execution {}", checkpoint_id, execution_id);

        self.cleanup_old_checkpoints(execution_id).await?;

        Ok(checkpoint_id)
    }

    pub async fn load_checkpoint(&self, checkpoint_id: &str) -> Result<Value> {
        let mut entries = fs::read_dir(&self.storage_path)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to read checkpoint directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to read entry: {}", e)))?
        {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.contains(checkpoint_id) {
                    let content = fs::read_to_string(&path)
                        .await
                        .map_err(|e| RomaError::StorageError(format!("Failed to read checkpoint file: {}", e)))?;

                    let checkpoint: Value = serde_json::from_str(&content)
                        .map_err(|e| RomaError::SerializationError(e.to_string()))?;

                    return Ok(checkpoint["dag_data"].clone());
                }
            }
        }

        Err(RomaError::StorageError(format!(
            "Checkpoint {} not found",
            checkpoint_id
        )))
    }

    pub async fn list_checkpoints(&self, execution_id: &str) -> Result<Vec<String>> {
        let mut checkpoints = Vec::new();
        let mut entries = fs::read_dir(&self.storage_path)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to read checkpoint directory: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to read entry: {}", e)))?
        {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(&format!("checkpoint_{}_", execution_id)) {
                    checkpoints.push(filename.to_string());
                }
            }
        }

        Ok(checkpoints)
    }

    async fn cleanup_old_checkpoints(&self, execution_id: &str) -> Result<()> {
        let mut checkpoints = self.list_checkpoints(execution_id).await?;

        if checkpoints.len() > self.max_checkpoints {
            checkpoints.sort();
            let to_remove = checkpoints.len() - self.max_checkpoints;

            for checkpoint in checkpoints.iter().take(to_remove) {
                let path = self.storage_path.join(checkpoint);
                fs::remove_file(&path)
                    .await
                    .map_err(|e| RomaError::StorageError(format!("Failed to remove old checkpoint: {}", e)))?;
                info!("Removed old checkpoint: {}", checkpoint);
            }
        }

        Ok(())
    }
}
