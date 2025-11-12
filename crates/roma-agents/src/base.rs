use async_trait::async_trait;
use roma_core::{Result, RomaError};
use roma_config::AgentConfig;
use std::sync::Arc;

#[async_trait]
pub trait BaseAgent: Send + Sync {
    async fn execute(&self, input: &str, context: Option<&str>) -> Result<String>;

    async fn execute_with_tools(
        &self,
        input: &str,
        context: Option<&str>,
        tools: Vec<Arc<dyn rig::tool::Tool>>,
    ) -> Result<String>;
}

pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    pub async fn build_openai_agent(
        &self,
    ) -> Result<rig::providers::openai::CompletionModel> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| RomaError::ConfigError("OPENAI_API_KEY not set".to_string()))?;

        let client = rig::providers::openai::Client::new(&api_key);

        let model = client
            .completion_model(&self.config.llm.model)
            .temperature(self.config.llm.temperature as f64)
            .max_tokens(self.config.llm.max_tokens as usize)
            .build();

        Ok(model)
    }

    pub async fn build_anthropic_agent(
        &self,
    ) -> Result<rig::providers::anthropic::CompletionModel> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| RomaError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;

        let client = rig::providers::anthropic::Client::new(&api_key);

        let model = client
            .completion_model(&self.config.llm.model)
            .temperature(self.config.llm.temperature as f64)
            .max_tokens(self.config.llm.max_tokens as usize)
            .build();

        Ok(model)
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

pub async fn execute_completion<M>(
    model: &M,
    input: &str,
    system_prompt: Option<&str>,
) -> Result<String>
where
    M: rig::completion::CompletionModel,
{
    let mut request = model.completion_request(input);

    if let Some(prompt) = system_prompt {
        request = request.preamble(prompt);
    }

    let response = request
        .send()
        .await
        .map_err(|e| RomaError::LlmError(format!("Completion failed: {}", e)))?;

    Ok(response.choice)
}
