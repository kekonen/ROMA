use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use roma_engine::RecursiveSolver;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Deserialize)]
pub struct CreateExecutionRequest {
    pub goal: String,
}

#[derive(Serialize)]
pub struct CreateExecutionResponse {
    pub execution_id: String,
    pub status: String,
}

pub async fn create_execution(
    State(state): State<AppState>,
    Json(req): Json<CreateExecutionRequest>,
) -> impl IntoResponse {
    info!("Creating new execution for goal: {}", req.goal);

    let solver = RecursiveSolver::new((*state.config).clone(), (*state.storage).clone());
    let goal = req.goal.clone();

    tokio::spawn(async move {
        match solver.solve(goal).await {
            Ok(result) => {
                info!("Execution {} completed successfully", result.task_id);
            }
            Err(e) => {
                error!("Execution failed: {}", e);
            }
        }
    });

    let execution_id = uuid::Uuid::new_v4().to_string();

    (
        StatusCode::CREATED,
        Json(CreateExecutionResponse {
            execution_id,
            status: "started".to_string(),
        }),
    )
}

#[derive(Serialize)]
pub struct ExecutionStatusResponse {
    pub execution_id: String,
    pub status: String,
    pub result: Option<String>,
}

pub async fn get_execution(
    State(state): State<AppState>,
    Path(execution_id): Path<String>,
) -> impl IntoResponse {
    if let Some(task) = state.executions.get(&execution_id) {
        (
            StatusCode::OK,
            Json(ExecutionStatusResponse {
                execution_id: task.task_id.clone(),
                status: format!("{:?}", task.status),
                result: task.result.clone(),
            }),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ExecutionStatusResponse {
                execution_id,
                status: "not_found".to_string(),
                result: None,
            }),
        )
    }
}

pub async fn get_dag(
    State(_state): State<AppState>,
    Path(execution_id): Path<String>,
) -> impl IntoResponse {
    // Get DAG structure for the execution
    // This would require storing DAGs in AppState
    info!("Fetching DAG for execution: {}", execution_id);

    (StatusCode::OK, Json(serde_json::json!({
        "execution_id": execution_id,
        "nodes": [],
        "edges": [],
        "message": "DAG structure would be returned here with proper state management"
    })))
}

pub async fn list_checkpoints(State(_state): State<AppState>) -> impl IntoResponse {
    // List all available checkpoints
    // This would require storing checkpoints in AppState or a database
    info!("Listing checkpoints");

    (StatusCode::OK, Json(serde_json::json!({
        "checkpoints": [],
        "message": "Checkpoint list would be returned here with proper state management"
    })))
}

pub async fn restore_checkpoint(
    State(_state): State<AppState>,
    Path(_checkpoint_id): Path<String>,
) -> impl IntoResponse {
    // Restore execution from a checkpoint
    // This would require checkpoint storage and restoration logic
    info!("Restoring checkpoint: {}", _checkpoint_id);

    (StatusCode::OK, Json(serde_json::json!({
        "checkpoint_id": _checkpoint_id,
        "status": "restored",
        "message": "Checkpoint restoration would be performed here with proper state management"
    })))
}

pub async fn get_metrics(State(_state): State<AppState>) -> impl IntoResponse {
    // Get execution metrics
    // This would require metrics collection and aggregation
    info!("Fetching metrics");

    (StatusCode::OK, Json(serde_json::json!({
        "executions": {
            "total": 0,
            "active": 0,
            "completed": 0,
            "failed": 0
        },
        "performance": {
            "avg_duration_ms": 0,
            "avg_tasks_per_execution": 0
        },
        "message": "Metrics would be calculated and returned here with proper metrics collection"
    })))
}

pub async fn get_traces(State(_state): State<AppState>) -> impl IntoResponse {
    // Get OpenTelemetry traces
    // This would require trace collection and export
    info!("Fetching traces");

    (StatusCode::OK, Json(serde_json::json!({
        "traces": [],
        "message": "Trace data would be returned here with proper observability integration"
    })))
}
