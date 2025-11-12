use async_trait::async_trait;
use roma_core::{Result, RomaError};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;

#[derive(Clone)]
pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub async fn new(connection_url: &str, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(connection_url)
            .await
            .map_err(|e| RomaError::StorageError(format!("Failed to connect to database: {}", e)))?;

        Ok(Self { pool })
    }

    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS executions (
                execution_id TEXT PRIMARY KEY,
                config JSONB NOT NULL,
                status TEXT NOT NULL,
                start_time TIMESTAMPTZ NOT NULL,
                end_time TIMESTAMPTZ,
                dag_snapshot JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to create executions table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS traces (
                trace_id TEXT PRIMARY KEY,
                execution_id TEXT NOT NULL REFERENCES executions(execution_id),
                span_data JSONB NOT NULL,
                parent_span TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to create traces table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                checkpoint_id TEXT PRIMARY KEY,
                execution_id TEXT NOT NULL REFERENCES executions(execution_id),
                dag_data JSONB NOT NULL,
                metadata JSONB,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_checkpoints_execution_id ON checkpoints(execution_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to create checkpoints table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS toolkit_metrics (
                metric_id SERIAL PRIMARY KEY,
                execution_id TEXT NOT NULL REFERENCES executions(execution_id),
                toolkit_name TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                start_time TIMESTAMPTZ NOT NULL,
                duration_ms BIGINT,
                success BOOLEAN NOT NULL,
                error TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_toolkit_metrics_execution_id ON toolkit_metrics(execution_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to create toolkit_metrics table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS event_traces (
                event_id TEXT PRIMARY KEY,
                execution_id TEXT NOT NULL REFERENCES executions(execution_id),
                event_type TEXT NOT NULL,
                task_id TEXT NOT NULL,
                data JSONB NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_event_traces_execution_id ON event_traces(execution_id);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to create event_traces table: {}", e)))?;

        Ok(())
    }

    pub async fn save_execution(
        &self,
        execution_id: &str,
        config: &serde_json::Value,
        status: &str,
        start_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO executions (execution_id, config, status, start_time)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (execution_id) DO UPDATE SET
                config = EXCLUDED.config,
                status = EXCLUDED.status,
                start_time = EXCLUDED.start_time
            "#,
        )
        .bind(execution_id)
        .bind(config)
        .bind(status)
        .bind(start_time)
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to save execution: {}", e)))?;

        Ok(())
    }

    pub async fn update_execution_status(
        &self,
        execution_id: &str,
        status: &str,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        dag_snapshot: Option<&serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE executions
            SET status = $2, end_time = $3, dag_snapshot = $4
            WHERE execution_id = $1
            "#,
        )
        .bind(execution_id)
        .bind(status)
        .bind(end_time)
        .bind(dag_snapshot)
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to update execution: {}", e)))?;

        Ok(())
    }

    pub async fn save_checkpoint(
        &self,
        checkpoint_id: &str,
        execution_id: &str,
        dag_data: &serde_json::Value,
        metadata: Option<&serde_json::Value>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO checkpoints (checkpoint_id, execution_id, dag_data, metadata)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(checkpoint_id)
        .bind(execution_id)
        .bind(dag_data)
        .bind(metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to save checkpoint: {}", e)))?;

        Ok(())
    }

    pub async fn load_checkpoint(&self, checkpoint_id: &str) -> Result<serde_json::Value> {
        let row = sqlx::query(
            r#"
            SELECT dag_data FROM checkpoints WHERE checkpoint_id = $1
            "#,
        )
        .bind(checkpoint_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to load checkpoint: {}", e)))?;

        let dag_data: serde_json::Value = row.try_get("dag_data")
            .map_err(|e| RomaError::StorageError(format!("Failed to extract dag_data: {}", e)))?;

        Ok(dag_data)
    }

    pub async fn save_toolkit_metric(
        &self,
        execution_id: &str,
        toolkit_name: &str,
        tool_name: &str,
        start_time: chrono::DateTime<chrono::Utc>,
        duration_ms: Option<i64>,
        success: bool,
        error: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO toolkit_metrics (execution_id, toolkit_name, tool_name, start_time, duration_ms, success, error)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(execution_id)
        .bind(toolkit_name)
        .bind(tool_name)
        .bind(start_time)
        .bind(duration_ms)
        .bind(success)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to save toolkit metric: {}", e)))?;

        Ok(())
    }

    pub async fn save_event(
        &self,
        event_id: &str,
        execution_id: &str,
        event_type: &str,
        task_id: &str,
        data: &serde_json::Value,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO event_traces (event_id, execution_id, event_type, task_id, data, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(event_id)
        .bind(execution_id)
        .bind(event_type)
        .bind(task_id)
        .bind(data)
        .bind(timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| RomaError::StorageError(format!("Failed to save event: {}", e)))?;

        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
