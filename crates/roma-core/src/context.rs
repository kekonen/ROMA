use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub storage_path: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExecutionContext {
    pub fn new(storage_path: String) -> Self {
        Self {
            execution_id: Uuid::new_v4().to_string(),
            storage_path,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn generate_agent_context(&self, task_goal: &str) -> String {
        format!(
            r#"<context>
<overall_objective>{}</overall_objective>
<execution_id>{}</execution_id>
<storage_path>{}</storage_path>
</context>"#,
            task_goal, self.execution_id, self.storage_path
        )
    }
}

#[derive(Debug, Clone)]
pub struct SharedContext {
    inner: Arc<RwLock<ContextData>>,
}

#[derive(Debug, Clone)]
struct ContextData {
    execution_context: ExecutionContext,
    event_buffer: Vec<Event>,
    metrics_buffer: Vec<crate::models::ToolInvocation>,
}

impl SharedContext {
    pub fn new(execution_context: ExecutionContext) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ContextData {
                execution_context,
                event_buffer: Vec::new(),
                metrics_buffer: Vec::new(),
            })),
        }
    }

    pub fn execution_context(&self) -> ExecutionContext {
        self.inner.read().execution_context.clone()
    }

    pub fn add_event(&self, event: Event) {
        self.inner.write().event_buffer.push(event);
    }

    pub fn add_metric(&self, metric: crate::models::ToolInvocation) {
        self.inner.write().metrics_buffer.push(metric);
    }

    pub fn drain_events(&self) -> Vec<Event> {
        self.inner.write().event_buffer.drain(..).collect()
    }

    pub fn drain_metrics(&self) -> Vec<crate::models::ToolInvocation> {
        self.inner.write().metrics_buffer.drain(..).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub event_type: EventType,
    pub task_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    TaskCreated,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    Atomized,
    Planned,
    Executed,
    Aggregated,
    Verified,
}

impl Event {
    pub fn new(event_type: EventType, task_id: String, data: serde_json::Value) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type,
            task_id,
            timestamp: chrono::Utc::now(),
            data,
        }
    }
}
