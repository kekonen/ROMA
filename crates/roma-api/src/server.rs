use axum::{
    routing::{get, post},
    Router,
};
use roma_config::RomaConfig;
use roma_core::Result;
use roma_storage::FileStorage;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::handlers::*;
use crate::state::AppState;

pub struct ApiServer {
    state: AppState,
    port: u16,
}

impl ApiServer {
    pub fn new(config: RomaConfig, storage: FileStorage, port: u16) -> Self {
        let state = AppState::new(config, storage);
        Self { state, port }
    }

    pub async fn run(self) -> Result<()> {
        let app = Router::new()
            .route("/health", get(health))
            .route("/api/v1/executions", post(create_execution))
            .route("/api/v1/executions/:id", get(get_execution))
            .route("/api/v1/executions/:id/dag", get(get_dag))
            .route("/api/v1/checkpoints", get(list_checkpoints))
            .route("/api/v1/checkpoints/:id/restore", post(restore_checkpoint))
            .route("/api/v1/metrics", get(get_metrics))
            .route("/api/v1/traces", get(get_traces))
            .layer(CorsLayer::permissive())
            .with_state(self.state);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!("Starting API server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| roma_core::RomaError::IoError(e))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| roma_core::RomaError::Unknown(e.to_string()))?;

        Ok(())
    }
}
