use roma_core::Result;
use roma_storage::ExecutionStorage;
use std::collections::HashMap;
use std::sync::Arc;

use crate::base::{DynTool, Toolkit};
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

    pub fn get_all_tools(&self) -> Vec<DynTool> {
        let mut all_tools = Vec::new();
        for toolkit in self.toolkits.values() {
            all_tools.extend(toolkit.tools());
        }
        all_tools
    }

    pub fn get_tools_by_names(&self, names: &[String]) -> Vec<DynTool> {
        let mut tools = Vec::new();
        for name in names {
            if let Some(toolkit) = self.toolkits.get(name) {
                tools.extend(toolkit.tools());
            }
        }
        tools
    }

    pub fn toolkit_names(&self) -> Vec<String> {
        self.toolkits.keys().cloned().collect()
    }
}

impl Default for ToolkitManager {
    fn default() -> Self {
        Self::new()
    }
}
