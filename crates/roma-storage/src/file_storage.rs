use async_trait::async_trait;
use roma_core::{Result, RomaError};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::Storage;

#[derive(Debug, Clone)]
pub struct FileStorage {
    pub base_path: PathBuf,
}

impl FileStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to create base directory: {}", e)))?;
        Ok(())
    }

    pub fn resolve_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path)
    }

    pub fn execution_storage(&self, execution_id: &str) -> ExecutionStorage {
        ExecutionStorage {
            base_storage: self.clone(),
            execution_id: execution_id.to_string(),
        }
    }
}

#[async_trait]
impl crate::Storage for FileStorage {
    async fn write(&self, path: &str, content: &[u8]) -> Result<()> {
        let full_path = self.resolve_path(path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| RomaError::StorageError(format!("Failed to create directory: {}", e)))?;
        }

        let mut file = fs::File::create(&full_path)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to create file: {}", e)))?;

        file.write_all(content)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.resolve_path(path);

        let mut file = fs::File::open(&full_path)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to open file: {}", e)))?;

        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to read file: {}", e)))?;

        Ok(content)
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.resolve_path(path);
        Ok(full_path.exists())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.resolve_path(path);

        if full_path.is_dir() {
            fs::remove_dir_all(&full_path)
                .await
                .map_err(|e| RomaError::StorageError(format!("Failed to delete directory: {}", e)))?;
        } else {
            fs::remove_file(&full_path)
                .await
                .map_err(|e| RomaError::StorageError(format!("Failed to delete file: {}", e)))?;
        }

        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let full_path = self.resolve_path(prefix);

        let mut entries = Vec::new();
        let mut read_dir = fs::read_dir(&full_path)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to read entry: {}", e)))?
        {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    entries.push(name_str.to_string());
                }
            }
        }

        Ok(entries)
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionStorage {
    base_storage: FileStorage,
    execution_id: String,
}

impl ExecutionStorage {
    pub async fn init(&self) -> Result<()> {
        let execution_path = format!("executions/{}", self.execution_id);
        let full_path = self.base_storage.resolve_path(&execution_path);

        fs::create_dir_all(&full_path)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to create execution directory: {}", e)))?;

        Ok(())
    }

    pub async fn write_artifact(&self, artifact_name: &str, content: &str) -> Result<()> {
        let path = format!("executions/{}/artifacts/{}", self.execution_id, artifact_name);
        self.base_storage.write(&path, content.as_bytes()).await
    }

    pub async fn read_artifact(&self, artifact_name: &str) -> Result<String> {
        let path = format!("executions/{}/artifacts/{}", self.execution_id, artifact_name);
        let content = self.base_storage.read(&path).await?;
        String::from_utf8(content)
            .map_err(|e| RomaError::StorageError(format!("Failed to decode artifact: {}", e)))
    }

    pub async fn list_artifacts(&self) -> Result<Vec<String>> {
        let path = format!("executions/{}/artifacts", self.execution_id);
        self.base_storage.list(&path).await
    }

    pub fn storage_path(&self) -> String {
        format!("{}/executions/{}", self.base_storage.base_path.display(), self.execution_id)
    }
}
