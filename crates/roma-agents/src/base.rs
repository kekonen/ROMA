use async_trait::async_trait;
use roma_core::{Result, RomaError};
use roma_config::AgentConfig;
use rig::client::CompletionClient;

#[async_trait]
pub trait BaseAgent: Send + Sync {
    async fn execute(&self, input: &str, context: Option<&str>) -> Result<String>;

    async fn execute_with_tools(
        &self,
        input: &str,
        context: Option<&str>,
        tools: Vec<String>, // Tool names for now, since rig::tool::Tool is not dyn-compatible
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
    ) -> Result<impl rig::completion::CompletionModel> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| RomaError::ConfigError("OPENAI_API_KEY not set".to_string()))?;

        let client = rig::providers::openai::Client::new(&api_key);

        // In rig-core 0.24.0, the model is returned directly without builder pattern
        let model = client.completion_model(&self.config.llm.model);

        Ok(model)
    }

    pub async fn build_anthropic_agent(
        &self,
    ) -> Result<impl rig::completion::CompletionModel> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| RomaError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;

        let client = rig::providers::anthropic::Client::new(&api_key);

        // In rig-core 0.24.0, the model is returned directly without builder pattern
        let model = client.completion_model(&self.config.llm.model);

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
        request = request.preamble(prompt.to_string());
    }

    let response = request
        .send()
        .await
        .map_err(|e| RomaError::LlmError(format!("Completion failed: {}", e)))?;

    // Convert response to string
    // response.choice is a complex type, let's just format it for now
    let text = format!("{:?}", response.choice);

    Ok(text)
}
