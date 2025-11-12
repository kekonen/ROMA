use thiserror::Error;

#[derive(Error, Debug)]
pub enum RomaError {
    #[error("Task execution failed: {0}")]
    ExecutionError(String),

    #[error("Planning failed: {0}")]
    PlanningError(String),

    #[error("Atomization failed: {0}")]
    AtomizationError(String),

    #[error("Aggregation failed: {0}")]
    AggregationError(String),

    #[error("Verification failed: {0}")]
    VerificationError(String),

    #[error("DAG error: {0}")]
    DagError(String),

    #[error("Cycle detected in DAG")]
    CycleDetected,

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),

    #[error("Maximum depth exceeded: {current} >= {max}")]
    MaxDepthExceeded { current: usize, max: usize },

    #[error("Timeout exceeded: {0}s")]
    TimeoutExceeded(u64),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Toolkit error: {0}")]
    ToolkitError(String),

    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, RomaError>;
