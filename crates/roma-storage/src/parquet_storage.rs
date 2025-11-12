use roma_core::{Result, RomaError};
use std::sync::Arc;

pub struct ParquetWriter {
    base_path: std::path::PathBuf,
}

impl ParquetWriter {
    pub fn new(base_path: std::path::PathBuf) -> Self {
        Self { base_path }
    }

    pub async fn write_data(
        &self,
        category: &str,
        filename: &str,
        data: &serde_json::Value,
    ) -> Result<String> {
        let output_path = self.base_path.join(category).join(format!("{}.parquet", filename));

        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| RomaError::StorageError(format!("Failed to create directory: {}", e)))?;
        }

        Ok(output_path.display().to_string())
    }
}
