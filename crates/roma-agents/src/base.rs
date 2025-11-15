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
        tools: Vec<String>, // Tool names (rig::tool::Tool is not dyn-compatible in rig-core 0.24.0)
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
    max_tokens: Option<u32>,
) -> Result<String>
where
    M: rig::completion::CompletionModel,
{
    let mut request = model.completion_request(input);

    if let Some(prompt) = system_prompt {
        request = request.preamble(prompt.to_string());
    }

    if let Some(tokens) = max_tokens {
        request = request.max_tokens(tokens as u64);
    }

    let response = request
        .send()
        .await
        .map_err(|e| RomaError::LlmError(format!("Completion failed: {}", e)))?;

    // Extract text from response.choice (OneOrMany<AssistantContent>)
    // OneOrMany is a struct with first() and rest() methods
    let first_text = extract_text_from_content(&response.choice.first());
    let rest_texts: Vec<String> = response.choice.rest()
        .iter()
        .map(extract_text_from_content)
        .collect();

    let text = if rest_texts.is_empty() {
        first_text
    } else {
        let mut all_texts = vec![first_text];
        all_texts.extend(rest_texts);
        all_texts.join("\n")
    };

    Ok(text)
}

fn extract_text_from_content(content: &rig::completion::message::AssistantContent) -> String {
    use rig::completion::message::AssistantContent;

    match content {
        AssistantContent::Text(text) => text.text().to_string(),
        AssistantContent::ToolCall(tool_call) => {
            format!("Tool call: {} (id: {})", tool_call.function.name, tool_call.id)
        }
        AssistantContent::Reasoning(reasoning) => {
            reasoning.reasoning.join("\n")
        }
    }
}
