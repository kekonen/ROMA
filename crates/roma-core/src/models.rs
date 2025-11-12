use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Ready,
    Executing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Plan,
    Execute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Retrieve,
    Write,
    Think,
    CodeInterpret,
    ImageGeneration,
}

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::Retrieve => "RETRIEVE",
            TaskType::Write => "WRITE",
            TaskType::Think => "THINK",
            TaskType::CodeInterpret => "CODE_INTERPRET",
            TaskType::ImageGeneration => "IMAGE_GENERATION",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub token_usage: Option<TokenUsage>,
    pub retry_count: u32,
    pub error_messages: Vec<String>,
}

impl Default for NodeMetrics {
    fn default() -> Self {
        Self {
            start_time: None,
            end_time: None,
            duration_ms: None,
            token_usage: None,
            retry_count: 0,
            error_messages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub task_id: String,
    pub goal: String,
    pub depth: usize,
    pub max_depth: usize,
    pub status: TaskStatus,
    pub node_type: NodeType,
    pub result: Option<String>,
    pub parent_id: Option<String>,
    pub subgraph_id: Option<String>,
    pub dependencies: Vec<String>,
    pub metrics: NodeMetrics,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TaskNode {
    pub fn new(goal: String, depth: usize, max_depth: usize) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            goal,
            depth,
            max_depth,
            status: TaskStatus::Pending,
            node_type: NodeType::Plan,
            result: None,
            parent_id: None,
            subgraph_id: None,
            dependencies: Vec::new(),
            metrics: NodeMetrics::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: String) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn start_execution(&mut self) {
        self.status = TaskStatus::Executing;
        self.metrics.start_time = Some(Utc::now());
    }

    pub fn complete_execution(&mut self, result: String) {
        self.status = TaskStatus::Completed;
        self.result = Some(result);
        self.metrics.end_time = Some(Utc::now());

        if let (Some(start), Some(end)) = (self.metrics.start_time, self.metrics.end_time) {
            self.metrics.duration_ms = Some((end - start).num_milliseconds() as u64);
        }
    }

    pub fn fail_execution(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.metrics.error_messages.push(error);
        self.metrics.end_time = Some(Utc::now());

        if let (Some(start), Some(end)) = (self.metrics.start_time, self.metrics.end_time) {
            self.metrics.duration_ms = Some((end - start).num_milliseconds() as u64);
        }
    }

    pub fn is_atomic(&self) -> bool {
        self.node_type == NodeType::Execute
    }

    pub fn is_ready(&self) -> bool {
        self.status == TaskStatus::Ready
    }

    pub fn is_completed(&self) -> bool {
        self.status == TaskStatus::Completed
    }

    pub fn is_failed(&self) -> bool {
        self.status == TaskStatus::Failed
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub task_id: String,
    pub goal: String,
    pub task_type: TaskType,
    pub dependencies: Vec<String>,
    pub result: Option<String>,
    pub context_input: Option<String>,
}

impl SubTask {
    pub fn new(goal: String, task_type: TaskType) -> Self {
        Self {
            task_id: Uuid::new_v4().to_string(),
            goal,
            task_type,
            dependencies: Vec::new(),
            result: None,
            context_input: None,
        }
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context_input = Some(context);
        self
    }

    pub fn into_task_node(self, depth: usize, max_depth: usize) -> TaskNode {
        TaskNode {
            task_id: self.task_id,
            goal: self.goal,
            depth,
            max_depth,
            status: if self.dependencies.is_empty() {
                TaskStatus::Ready
            } else {
                TaskStatus::Pending
            },
            node_type: NodeType::Execute,
            result: self.result,
            parent_id: None,
            subgraph_id: None,
            dependencies: self.dependencies,
            metrics: NodeMetrics::default(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("task_type".to_string(), serde_json::json!(self.task_type.as_str()));
                if let Some(context) = self.context_input {
                    map.insert("context_input".to_string(), serde_json::json!(context));
                }
                map
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomizationResult {
    pub is_atomic: bool,
    pub node_type: NodeType,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningResult {
    pub subtasks: Vec<SubTask>,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub output: String,
    pub sources: Vec<String>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub synthesized_output: String,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub verdict: bool,
    pub feedback: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: String,
    pub artifact_type: ArtifactType,
    pub name: String,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    Code,
    Document,
    Data,
    Image,
    Other,
}

impl Artifact {
    pub fn new(artifact_type: ArtifactType, name: String, content: String) -> Self {
        Self {
            artifact_id: Uuid::new_v4().to_string(),
            artifact_type,
            name,
            content,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub tool_name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub success: bool,
    pub error: Option<String>,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
}

impl ToolInvocation {
    pub fn new(tool_name: String, input: serde_json::Value) -> Self {
        Self {
            tool_name,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            success: false,
            error: None,
            input,
            output: None,
        }
    }

    pub fn complete(&mut self, output: serde_json::Value) {
        self.success = true;
        self.output = Some(output);
        self.end_time = Some(Utc::now());
        self.duration_ms = Some((self.end_time.unwrap() - self.start_time).num_milliseconds() as u64);
    }

    pub fn fail(&mut self, error: String) {
        self.success = false;
        self.error = Some(error);
        self.end_time = Some(Utc::now());
        self.duration_ms = Some((self.end_time.unwrap() - self.start_time).num_milliseconds() as u64);
    }
}
