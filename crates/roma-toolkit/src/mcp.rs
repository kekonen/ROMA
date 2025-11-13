// MCP toolkit is currently disabled as mcp-rig is not available on crates.io
// To enable MCP support, implement a custom MCP client integration

use async_trait::async_trait;
use roma_core::Result;

use crate::base::{DynTool, Toolkit};

pub struct McpToolkit;

impl McpToolkit {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Toolkit for McpToolkit {
    fn name(&self) -> &str {
        "mcp"
    }

    fn tools(&self) -> Vec<DynTool> {
        // MCP tools would be dynamically loaded from MCP servers
        Vec::new()
    }

    async fn setup(&mut self) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}
