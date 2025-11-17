use roma_core::{ExecutionContext, Result, RomaError, SharedContext, TaskNode, NodeType, TaskStatus, Event, EventType};
use roma_config::RomaConfig;
use roma_agents::{AgentFactory, Agents};
use roma_storage::{FileStorage, ExecutionStorage};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error, debug};

use crate::{TaskDag, EventLoopScheduler};

pub struct RecursiveSolver {
    config: RomaConfig,
    agents: Agents,
    storage: FileStorage,
    event_loop: EventLoopScheduler,
}

impl RecursiveSolver {
    pub fn new(config: RomaConfig, storage: FileStorage) -> Self {
        let agent_factory = AgentFactory::new(config.agents.clone());
        let agents = agent_factory.create_all();
        let event_loop = EventLoopScheduler::new(config.runtime.max_concurrent_tasks);

        Self {
            config,
            agents,
            storage,
            event_loop,
        }
    }

    pub async fn solve(&self, goal: String) -> Result<TaskNode> {
        let dag = TaskDag::new();

        let execution_context = ExecutionContext::new(self.storage.base_path.display().to_string());
        let exec_storage = self.storage.execution_storage(&execution_context.execution_id);
        exec_storage.init().await?;

        info!(
            "Starting execution {} for goal: {}",
            execution_context.execution_id, goal
        );

        let shared_context = SharedContext::new(execution_context.clone());

        let root_task = TaskNode::new(goal.clone(), 0, self.config.runtime.max_depth);
        let root_id = dag.add_task(root_task.clone())?;

        let result = self
            .async_solve(&root_id, &dag, 0, &shared_context, &exec_storage)
            .await?;

        info!("Execution {} completed", execution_context.execution_id);

        Ok(result)
    }

    async fn async_solve(
        &self,
        task_id: &str,
        dag: &TaskDag,
        depth: usize,
        context: &SharedContext,
        storage: &ExecutionStorage,
    ) -> Result<TaskNode> {
        let mut task = dag.get_task(task_id)?;

        if depth >= self.config.runtime.max_depth {
            warn!(
                "Max depth {} reached for task {}, forcing execution",
                self.config.runtime.max_depth, task_id
            );
            return self.force_execute(task, context, storage).await;
        }

        debug!("Processing task {} at depth {}", task_id, depth);

        let atomization_result = self
            .agents
            .atomizer
            .atomize(
                &task.goal,
                Some(&context.execution_context().generate_agent_context(&task.goal)),
            )
            .await?;

        context.add_event(Event::new(
            EventType::Atomized,
            task_id.to_string(),
            serde_json::json!({
                "is_atomic": atomization_result.is_atomic,
                "reasoning": atomization_result.reasoning,
            }),
        ));

        dag.update_task(task_id, |t| {
            t.node_type = atomization_result.node_type;
        })?;

        task = dag.get_task(task_id)?;

        if task.node_type == NodeType::Execute {
            self.execute_atomic_task(task, context, storage).await
        } else {
            self.decompose_and_execute(task_id, dag, depth, context, storage)
                .await
        }
    }

    async fn decompose_and_execute(
        &self,
        task_id: &str,
        dag: &TaskDag,
        depth: usize,
        context: &SharedContext,
        storage: &ExecutionStorage,
    ) -> Result<TaskNode> {
        let task = dag.get_task(task_id)?;

        debug!("Planning task {}", task_id);

        let planning_result = self
            .agents
            .planner
            .plan(
                &task.goal,
                Some(&context.execution_context().generate_agent_context(&task.goal)),
            )
            .await?;

        context.add_event(Event::new(
            EventType::Planned,
            task_id.to_string(),
            serde_json::json!({
                "subtask_count": planning_result.subtasks.len(),
                "reasoning": planning_result.reasoning,
            }),
        ));

        let subgraph = TaskDag::new();
        let subgraph_id = format!("{}_subgraph", task_id);

        let mut task_id_map = std::collections::HashMap::new();

        for subtask in &planning_result.subtasks {
            let task_node = subtask.clone().into_task_node(depth + 1, self.config.runtime.max_depth);
            let new_task_id = subgraph.add_task(task_node)?;
            task_id_map.insert(subtask.task_id.clone(), new_task_id);
        }

        for subtask in &planning_result.subtasks {
            let new_task_id = task_id_map.get(&subtask.task_id).unwrap();
            for dep_id in &subtask.dependencies {
                if let Some(new_dep_id) = task_id_map.get(dep_id) {
                    subgraph.add_dependency(new_dep_id, new_task_id)?;
                }
            }
        }

        dag.add_subgraph(subgraph_id.clone(), subgraph.clone());

        info!(
            "Executing {} subtasks for task {}",
            planning_result.subtasks.len(),
            task_id
        );

        let context_arc = Arc::new(context.clone());
        let storage_arc = Arc::new(storage.clone());
        let dag_clone = dag.clone();
        let subgraph_clone = subgraph.clone();
        let depth_clone = depth;

        self.event_loop
            .execute_with_dependencies(&subgraph, move |subtask| {
                let _context = context_arc.clone();
                let _storage = storage_arc.clone();
                let _subgraph = subgraph_clone.clone();

                async move {
                    // Placeholder - need to implement proper task execution with self reference
                    // For now, return the task as-is
                    Ok(subtask.clone())
                }
            })
            .await?;

        let completed_subtasks: Vec<_> = planning_result
            .subtasks
            .iter()
            .map(|st| {
                let task_id = task_id_map.get(&st.task_id).unwrap();
                let mut subtask = st.clone();
                if let Ok(completed) = subgraph.get_task(task_id) {
                    subtask.result = completed.result.clone();
                }
                subtask
            })
            .collect();

        debug!("Aggregating results for task {}", task_id);

        let aggregation_result = self
            .agents
            .aggregator
            .aggregate(
                &task.goal,
                &completed_subtasks,
                Some(&context.execution_context().generate_agent_context(&task.goal)),
            )
            .await?;

        context.add_event(Event::new(
            EventType::Aggregated,
            task_id.to_string(),
            serde_json::json!({
                "reasoning": aggregation_result.reasoning,
            }),
        ));

        dag.update_task(task_id, |t| {
            t.complete_execution(aggregation_result.synthesized_output.clone());
        })?;

        let final_task = dag.get_task(task_id)?;

        debug!("Verifying result for task {}", task_id);

        let verification_result = self
            .agents
            .verifier
            .verify(
                &task.goal,
                final_task.result.as_deref().unwrap_or(""),
                Some(&context.execution_context().generate_agent_context(&task.goal)),
            )
            .await?;

        context.add_event(Event::new(
            EventType::Verified,
            task_id.to_string(),
            serde_json::json!({
                "verdict": verification_result.verdict,
                "confidence": verification_result.confidence,
                "feedback": verification_result.feedback,
            }),
        ));

        if !verification_result.verdict {
            warn!(
                "Verification failed for task {} (confidence: {}): {:?}",
                task_id, verification_result.confidence, verification_result.feedback
            );
        }

        dag.get_task(task_id)
    }

    async fn execute_atomic_task(
        &self,
        mut task: TaskNode,
        context: &SharedContext,
        storage: &ExecutionStorage,
    ) -> Result<TaskNode> {
        debug!("Executing atomic task {}", task.task_id);

        task.start_execution();

        let result = self
            .agents
            .executor
            .execute_task(
                &task.goal,
                Some(&context.execution_context().generate_agent_context(&task.goal)),
            )
            .await?;

        context.add_event(Event::new(
            EventType::Executed,
            task.task_id.clone(),
            serde_json::json!({
                "output_length": result.output.len(),
            }),
        ));

        task.complete_execution(result.output);

        Ok(task)
    }

    async fn force_execute(
        &self,
        task: TaskNode,
        context: &SharedContext,
        storage: &ExecutionStorage,
    ) -> Result<TaskNode> {
        warn!("Force executing task {} due to max depth", task.task_id);
        self.execute_atomic_task(task, context, storage).await
    }
}
