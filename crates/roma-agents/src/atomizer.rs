use async_trait::async_trait;
use roma_core::{AtomizationResult, NodeType, Result, RomaError};
use roma_config::AgentConfig;

use crate::base::{execute_completion_placeholder, format_prompt_with_context, AgentBuilder, BaseAgent};

pub struct Atomizer {
    builder: AgentBuilder,
}

impl Atomizer {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            builder: AgentBuilder::new(config),
        }
    }

    const SYSTEM_PROMPT: &'static str = r#"You are an expert task analyzer. Your job is to determine if a task is atomic (directly executable) or needs to be broken down into subtasks.

An atomic task is one that can be completed in a single step without decomposition. It should be:
- Clear and specific
- Achievable with available tools
- Not requiring multiple distinct sub-goals

A non-atomic task requires decomposition when it:
- Contains multiple distinct objectives
- Requires sequential or parallel steps
- Needs coordination of different capabilities

Respond in the following JSON format:
{
  "is_atomic": true/false,
  "node_type": "EXECUTE" or "PLAN",
  "reasoning": "Brief explanation of your decision"
}

Be conservative: when in doubt, mark as non-atomic to ensure proper decomposition."#;

    pub async fn atomize(&self, goal: &str, context: Option<&str>) -> Result<AtomizationResult> {
        let prompt = format_prompt_with_context(
            &format!("Analyze this task:\n\n{}", goal),
            context,
        );

        let response = execute_completion_placeholder(&prompt, Some(Self::SYSTEM_PROMPT)).await?;

        self.parse_response(&response)
    }

    fn parse_response(&self, response: &str) -> Result<AtomizationResult> {
        let cleaned = response.trim();
        let cleaned = if cleaned.starts_with("```json") {
            cleaned
                .strip_prefix("```json")
                .unwrap_or(cleaned)
                .strip_suffix("```")
                .unwrap_or(cleaned)
                .trim()
        } else if cleaned.starts_with("```") {
            cleaned
                .strip_prefix("```")
                .unwrap_or(cleaned)
                .strip_suffix("```")
                .unwrap_or(cleaned)
                .trim()
        } else {
            cleaned
        };

        let parsed: serde_json::Value = serde_json::from_str(cleaned)
            .map_err(|e| RomaError::AtomizationError(format!("Failed to parse response: {}", e)))?;

        let is_atomic = parsed["is_atomic"]
            .as_bool()
            .ok_or_else(|| RomaError::AtomizationError("Missing 'is_atomic' field".to_string()))?;

        let node_type = if is_atomic {
            NodeType::Execute
        } else {
            NodeType::Plan
        };

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        Ok(AtomizationResult {
            is_atomic,
            node_type,
            reasoning,
        })
    }
}

#[async_trait]
impl BaseAgent for Atomizer {
    async fn execute(&self, input: &str, context: Option<&str>) -> Result<String> {
        let result = self.atomize(input, context).await?;
        serde_json::to_string(&result)
            .map_err(|e| RomaError::SerializationError(e.to_string()))
    }
}
