use async_trait::async_trait;
use roma_core::{AggregationResult, Result, RomaError, SubTask};
use roma_config::AgentConfig;

use crate::base::{execute_completion, format_prompt_with_context, AgentBuilder, BaseAgent};

pub struct Aggregator {
    builder: AgentBuilder,
}

impl Aggregator {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            builder: AgentBuilder::new(config),
        }
    }

    const SYSTEM_PROMPT: &'static str = r#"You are an expert at synthesizing information from multiple sources. Your job is to combine results from subtasks into a coherent, comprehensive response.

Guidelines:
1. Integrate all subtask results into a unified output
2. Resolve any contradictions or inconsistencies
3. Ensure the final output directly addresses the original goal
4. Maintain clarity and coherence
5. Highlight key findings and insights

Provide a well-structured synthesis that answers the original goal completely.

Respond in the following JSON format:
{
  "synthesized_output": "The comprehensive final output",
  "reasoning": "Brief explanation of how you synthesized the results"
}"#;

    pub async fn aggregate(
        &self,
        original_goal: &str,
        subtask_results: &[SubTask],
        context: Option<&str>,
    ) -> Result<AggregationResult> {
        let results_summary = subtask_results
            .iter()
            .enumerate()
            .map(|(i, subtask)| {
                format!(
                    "Subtask {}: {}\nResult: {}",
                    i + 1,
                    subtask.goal,
                    subtask.result.as_deref().unwrap_or("<no result>")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let prompt = format_prompt_with_context(
            &format!(
                "Original Goal: {}\n\nSubtask Results:\n{}\n\nSynthesize these results into a comprehensive response to the original goal.",
                original_goal, results_summary
            ),
            context,
        );

        let provider = &self.builder.config().llm.provider;
        let response = match provider.as_str() {
            "openai" => {
                let model = self.builder.build_openai_agent().await?;
                execute_completion(&model, &prompt, Some(Self::SYSTEM_PROMPT)).await?
            }
            "anthropic" => {
                let model = self.builder.build_anthropic_agent().await?;
                execute_completion(&model, &prompt, Some(Self::SYSTEM_PROMPT)).await?
            }
            _ => {
                return Err(RomaError::ConfigError(format!(
                    "Unsupported provider: {}",
                    provider
                )))
            }
        };

        self.parse_response(&response)
    }

    fn parse_response(&self, response: &str) -> Result<AggregationResult> {
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
            .map_err(|e| RomaError::AggregationError(format!("Failed to parse response: {}", e)))?;

        let synthesized_output = parsed["synthesized_output"]
            .as_str()
            .ok_or_else(|| RomaError::AggregationError("Missing 'synthesized_output' field".to_string()))?
            .to_string();

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        Ok(AggregationResult {
            synthesized_output,
            reasoning,
        })
    }
}

#[async_trait]
impl BaseAgent for Aggregator {
    async fn execute(&self, input: &str, context: Option<&str>) -> Result<String> {
        Err(RomaError::ExecutionError(
            "Aggregator requires subtask results".to_string(),
        ))
    }

    async fn execute_with_tools(
        &self,
        input: &str,
        context: Option<&str>,
        _tools: Vec<String>,
    ) -> Result<String> {
        self.execute(input, context).await
    }
}
