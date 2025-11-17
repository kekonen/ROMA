use async_trait::async_trait;
use roma_core::{Result, RomaError};
use roma_config::AgentConfig;
use std::sync::Arc;

#[async_trait]
pub trait BaseAgent: Send + Sync {
    async fn execute(&self, input: &str, context: Option<&str>) -> Result<String>;
}

pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    pub async fn build_openai_client(&self) -> Result<rig::providers::openai::Client> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| RomaError::ConfigError("OPENAI_API_KEY not set".to_string()))?;

        Ok(rig::providers::openai::Client::new(&api_key))
    }

    pub async fn build_anthropic_client(&self) -> Result<rig::providers::anthropic::Client> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| RomaError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;

        Ok(rig::providers::anthropic::Client::new(&api_key))
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
}

pub fn format_prompt_with_context(input: &str, context: Option<&str>) -> String {
    if let Some(ctx) = context {
        format!("{}\n\n{}", ctx, input)
    } else {
        input.to_string()
    }
}

// Placeholder for completion execution - needs to be implemented with rig-core 0.24.0 API
pub async fn execute_completion_placeholder(
    _input: &str,
    _system_prompt: Option<&str>,
) -> Result<String> {
    // TODO: Implement with rig-core 0.24.0 API
    Ok("Placeholder response - rig-core 0.24.0 API integration needed".to_string())
}
