use async_trait::async_trait;
use roma_core::{Result, RomaError, VerificationResult};
use roma_config::AgentConfig;

use crate::base::{execute_completion_placeholder, format_prompt_with_context, AgentBuilder, BaseAgent};

pub struct Verifier {
    builder: AgentBuilder,
}

impl Verifier {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            builder: AgentBuilder::new(config),
        }
    }

    const SYSTEM_PROMPT: &'static str = r#"You are an expert validator. Your job is to verify that an output satisfies the original goal.

Evaluation Criteria:
1. Completeness: Does the output fully address the goal?
2. Correctness: Is the information accurate and valid?
3. Coherence: Is the output well-structured and clear?
4. Relevance: Does it stay on topic and answer what was asked?

Respond in the following JSON format:
{
  "verdict": true/false,
  "feedback": "Specific feedback on what's good or what's missing (optional, especially if false)",
  "confidence": 0.0-1.0
}

Be rigorous but fair. Only mark as true if the output genuinely satisfies the goal."#;

    pub async fn verify(
        &self,
        goal: &str,
        candidate_output: &str,
        context: Option<&str>,
    ) -> Result<VerificationResult> {
        let prompt = format_prompt_with_context(
            &format!(
                "Goal: {}\n\nCandidate Output:\n{}\n\nVerify if this output satisfies the goal.",
                goal, candidate_output
            ),
            context,
        );

        let response = execute_completion_placeholder(&prompt, Some(Self::SYSTEM_PROMPT)).await?;

        self.parse_response(&response)
    }

    fn parse_response(&self, response: &str) -> Result<VerificationResult> {
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
            .map_err(|e| RomaError::VerificationError(format!("Failed to parse response: {}", e)))?;

        let verdict = parsed["verdict"]
            .as_bool()
            .ok_or_else(|| RomaError::VerificationError("Missing 'verdict' field".to_string()))?;

        let feedback = parsed["feedback"].as_str().map(String::from);

        let confidence = parsed["confidence"]
            .as_f64()
            .unwrap_or(0.5) as f32;

        Ok(VerificationResult {
            verdict,
            feedback,
            confidence,
        })
    }
}

#[async_trait]
impl BaseAgent for Verifier {
    async fn execute(&self, _input: &str, _context: Option<&str>) -> Result<String> {
        Err(RomaError::ExecutionError(
            "Verifier requires goal and candidate output".to_string(),
        ))
    }
}
