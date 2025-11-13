use async_trait::async_trait;
use roma_core::{ExecutionResult, Result, RomaError};
use roma_config::AgentConfig;

use crate::base::{format_prompt_with_context, AgentBuilder, BaseAgent};

pub struct Executor {
    builder: AgentBuilder,
}

impl Executor {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            builder: AgentBuilder::new(config),
        }
    }

    const SYSTEM_PROMPT: &'static str = r#"You are an expert task executor with access to various tools. Your job is to accomplish the given task using the available tools.

Process:
1. Analyze the task requirements
2. Identify which tools are needed
3. Execute the necessary tool calls
4. Synthesize the results into a clear output

Respond with the task results, including:
- The output/answer to the task
- Any sources or references used
- Any artifacts created (code, documents, data files)

Be thorough and precise. If a task cannot be completed with available tools, explain why and suggest alternatives."#;

    pub async fn execute_task(
        &self,
        goal: &str,
        context: Option<&str>,
        _tool_names: Vec<String>, // TODO: Implement custom tool integration
    ) -> Result<ExecutionResult> {
        let prompt = format_prompt_with_context(
            &format!("Execute this task:\n\n{}", goal),
            context,
        );

        let provider = &self.builder.config().llm.provider;
        let response = match provider.as_str() {
            "openai" => {
                let model = self.builder.build_openai_agent().await?;
                // Use the execute_completion function since rig-core 0.24.0 API has changed
                crate::base::execute_completion(&model, &prompt, Some(Self::SYSTEM_PROMPT)).await?
            }
            "anthropic" => {
                let model = self.builder.build_anthropic_agent().await?;
                // Use the execute_completion function since rig-core 0.24.0 API has changed
                crate::base::execute_completion(&model, &prompt, Some(Self::SYSTEM_PROMPT)).await?
            }
            _ => {
                return Err(RomaError::ConfigError(format!(
                    "Unsupported provider: {}",
                    provider
                )))
            }
        };

        Ok(ExecutionResult {
            output: response,
            sources: Vec::new(),
            artifacts: Vec::new(),
        })
    }
}

#[async_trait]
impl BaseAgent for Executor {
    async fn execute(&self, input: &str, context: Option<&str>) -> Result<String> {
        let result = self.execute_task(input, context, Vec::new()).await?;
        Ok(result.output)
    }

    async fn execute_with_tools(
        &self,
        input: &str,
        context: Option<&str>,
        tools: Vec<String>,
    ) -> Result<String> {
        let result = self.execute_task(input, context, tools).await?;
        Ok(result.output)
    }
}
