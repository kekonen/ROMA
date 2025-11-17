pub mod file_storage;

#[cfg(feature = "postgres")]
pub mod postgres_storage;

#[cfg(feature = "parquet-storage")]
pub mod parquet_storage;

pub use file_storage::{ExecutionStorage, FileStorage};

#[cfg(feature = "postgres")]
pub use postgres_storage::PostgresStorage;

use async_trait::async_trait;
use roma_core::Result;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn write(&self, path: &str, content: &[u8]) -> Result<()>;
    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    async fn exists(&self, path: &str) -> Result<bool>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}
