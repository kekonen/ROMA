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
    State(state): State<AppState>,
    Path(execution_id): Path<String>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({
        "error": "Not implemented"
    })))
}

pub async fn list_checkpoints(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({
        "error": "Not implemented"
    })))
}

pub async fn restore_checkpoint(
    State(state): State<AppState>,
    Path(checkpoint_id): Path<String>,
) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({
        "error": "Not implemented"
    })))
}

pub async fn get_metrics(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({
        "error": "Not implemented"
    })))
}

pub async fn get_traces(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, Json(serde_json::json!({
        "error": "Not implemented"
    })))
}
