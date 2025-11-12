use dashmap::DashMap;
use roma_config::RomaConfig;
use roma_core::TaskNode;
use roma_storage::FileStorage;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<RomaConfig>,
    pub storage: Arc<FileStorage>,
    pub executions: Arc<DashMap<String, TaskNode>>,
}

impl AppState {
    pub fn new(config: RomaConfig, storage: FileStorage) -> Self {
        Self {
            config: Arc::new(config),
            storage: Arc::new(storage),
            executions: Arc::new(DashMap::new()),
        }
    }
}
