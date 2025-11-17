use async_trait::async_trait;
use roma_core::{Result, ToolInvocation};

#[async_trait]
pub trait Toolkit: Send + Sync {
    fn name(&self) -> &str;

    async fn setup(&mut self) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    fn track_invocation(&self, _invocation: ToolInvocation) {
        // Default implementation does nothing
    }
}

pub fn build_success_response(data: serde_json::Value) -> String {
    serde_json::json!({
        "success": true,
        "data": data,
    })
    .to_string()
}

pub fn build_error_response(error: &str) -> String {
    serde_json::json!({
        "success": false,
        "error": error,
    })
    .to_string()
}
