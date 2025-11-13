use async_trait::async_trait;
use roma_core::{Result, ToolInvocation};
use std::sync::Arc;

/// Error type for tools that implements std::error::Error
#[derive(Debug, Clone)]
pub struct ToolError(pub String);

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolError {}

/// Type-erased tool wrapper since rig::tool::Tool is not dyn-compatible
pub struct DynTool {
    name: String,
    definition_fn: Arc<dyn Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = rig::completion::ToolDefinition> + Send>> + Send + Sync>,
    call_fn: Arc<dyn Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<String, String>> + Send>> + Send + Sync>,
}

impl DynTool {
    pub fn new<T: rig::tool::Tool + Clone + Send + Sync + 'static>(tool: T) -> Self
    where
        T::Args: serde::de::DeserializeOwned,
        T::Output: ToString,
        T::Error: ToString,
    {
        let tool_for_def = tool.clone();
        let tool_for_call = tool;

        Self {
            name: T::NAME.to_string(),
            definition_fn: Arc::new(move |prompt: String| {
                let tool = tool_for_def.clone();
                Box::pin(async move {
                    tool.definition(prompt).await
                })
            }),
            call_fn: Arc::new(move |args_json: String| {
                let tool = tool_for_call.clone();
                Box::pin(async move {
                    let args: T::Args = serde_json::from_str(&args_json)
                        .map_err(|e| format!("Failed to deserialize args: {}", e))?;
                    tool.call(args).await
                        .map(|output| output.to_string())
                        .map_err(|e| e.to_string())
                })
            }),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn definition(&self, prompt: String) -> rig::completion::ToolDefinition {
        (self.definition_fn)(prompt).await
    }

    pub async fn call(&self, args_json: String) -> std::result::Result<String, String> {
        (self.call_fn)(args_json).await
    }
}

#[async_trait]
pub trait Toolkit: Send + Sync {
    fn name(&self) -> &str;

    fn tools(&self) -> Vec<DynTool>;

    async fn setup(&mut self) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }

    fn track_invocation(&self, _invocation: ToolInvocation) {
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
