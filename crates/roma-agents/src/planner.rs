use async_trait::async_trait;
use roma_core::{PlanningResult, Result, RomaError, SubTask, TaskType};
use roma_config::AgentConfig;

use crate::base::{execute_completion, format_prompt_with_context, AgentBuilder, BaseAgent};

pub struct Planner {
    builder: AgentBuilder,
}

impl Planner {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            builder: AgentBuilder::new(config),
        }
    }

    const SYSTEM_PROMPT: &'static str = r#"You are an expert task decomposition planner. Your job is to break down complex tasks into subtasks with clear dependencies.

Task Types (MECE Framework):
- RETRIEVE: External data acquisition (API calls, web searches, database queries)
- WRITE: Content generation and synthesis (documents, code, reports)
- THINK: Analysis, reasoning, decision making (evaluation, planning, reflection)
- CODE_INTERPRET: Code execution and data processing (running scripts, transformations)
- IMAGE_GENERATION: Visual content creation

Guidelines:
1. Create clear, specific subtask goals
2. Identify dependencies between subtasks (tasks that must complete before others)
3. Maximize parallelism where possible
4. Each subtask should be atomic and achievable
5. Use dependency IDs to reference previous subtasks

Respond in the following JSON format:
{
  "subtasks": [
    {
      "task_id": "task_1",
      "goal": "Specific goal description",
      "task_type": "RETRIEVE|WRITE|THINK|CODE_INTERPRET|IMAGE_GENERATION",
      "dependencies": []
    },
    {
      "task_id": "task_2",
      "goal": "Another goal that depends on task_1",
      "task_type": "THINK",
      "dependencies": ["task_1"]
    }
  ],
  "reasoning": "Brief explanation of the decomposition strategy"
}"#;

    pub async fn plan(&self, goal: &str, context: Option<&str>) -> Result<PlanningResult> {
        let prompt = format_prompt_with_context(
            &format!("Decompose this task into subtasks:\n\n{}", goal),
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

    fn parse_response(&self, response: &str) -> Result<PlanningResult> {
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
            .map_err(|e| RomaError::PlanningError(format!("Failed to parse response: {}", e)))?;

        let subtasks_json = parsed["subtasks"]
            .as_array()
            .ok_or_else(|| RomaError::PlanningError("Missing 'subtasks' field".to_string()))?;

        let mut subtasks = Vec::new();
        for subtask_json in subtasks_json {
            let goal = subtask_json["goal"]
                .as_str()
                .ok_or_else(|| RomaError::PlanningError("Missing 'goal' field".to_string()))?
                .to_string();

            let task_type_str = subtask_json["task_type"]
                .as_str()
                .ok_or_else(|| RomaError::PlanningError("Missing 'task_type' field".to_string()))?;

            let task_type = match task_type_str {
                "RETRIEVE" => TaskType::Retrieve,
                "WRITE" => TaskType::Write,
                "THINK" => TaskType::Think,
                "CODE_INTERPRET" => TaskType::CodeInterpret,
                "IMAGE_GENERATION" => TaskType::ImageGeneration,
                _ => {
                    return Err(RomaError::PlanningError(format!(
                        "Invalid task_type: {}",
                        task_type_str
                    )))
                }
            };

            let dependencies = subtask_json["dependencies"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let task_id = subtask_json["task_id"]
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            let mut subtask = SubTask::new(goal, task_type);
            subtask.task_id = task_id;
            subtask.dependencies = dependencies;

            subtasks.push(subtask);
        }

        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        Ok(PlanningResult {
            subtasks,
            reasoning,
        })
    }
}

#[async_trait]
impl BaseAgent for Planner {
    async fn execute(&self, input: &str, context: Option<&str>) -> Result<String> {
        let result = self.plan(input, context).await?;
        serde_json::to_string(&result)
            .map_err(|e| RomaError::SerializationError(e.to_string()))
    }

    async fn execute_with_tools(
        &self,
        input: &str,
        context: Option<&str>,
        _tools: Vec<std::sync::Arc<dyn rig::tool::Tool>>,
    ) -> Result<String> {
        self.execute(input, context).await
    }
}
