use roma_core::Result;
use roma_storage::ExecutionStorage;
use std::collections::HashMap;
use std::sync::Arc;

use crate::base::Toolkit;
use crate::{ArtifactToolkit, CalculatorToolkit, FileToolkit};

#[cfg(feature = "docker")]
use crate::DockerToolkit;

pub struct ToolkitManager {
    toolkits: HashMap<String, Box<dyn Toolkit>>,
}

impl ToolkitManager {
    pub fn new() -> Self {
        Self {
            toolkits: HashMap::new(),
        }
    }

    pub fn with_execution_storage(storage: Arc<ExecutionStorage>) -> Self {
        let mut manager = Self::new();

        manager.register_toolkit(Box::new(FileToolkit::new(storage.clone())));
        manager.register_toolkit(Box::new(ArtifactToolkit::new(storage.clone())));
        manager.register_toolkit(Box::new(CalculatorToolkit::new()));

        #[cfg(feature = "docker")]
        if let Ok(docker_toolkit) = DockerToolkit::new() {
            manager.register_toolkit(Box::new(docker_toolkit));
        }

        manager
    }

    pub fn register_toolkit(&mut self, toolkit: Box<dyn Toolkit>) {
        let name = toolkit.name().to_string();
        self.toolkits.insert(name, toolkit);
    }

    pub async fn setup_all(&mut self) -> Result<()> {
        for toolkit in self.toolkits.values_mut() {
            toolkit.setup().await?;
        }
        Ok(())
    }

    pub async fn cleanup_all(&mut self) -> Result<()> {
        for toolkit in self.toolkits.values_mut() {
            toolkit.cleanup().await?;
        }
        Ok(())
    }

    pub fn toolkit_names(&self) -> Vec<String> {
        self.toolkits.keys().cloned().collect()
    }

    pub fn get_toolkit(&self, name: &str) -> Option<&Box<dyn Toolkit>> {
        self.toolkits.get(name)
    }

    pub fn get_toolkit_mut(&mut self, name: &str) -> Option<&mut Box<dyn Toolkit>> {
        self.toolkits.get_mut(name)
    }
}

impl Default for ToolkitManager {
    fn default() -> Self {
        Self::new()
    }
}
