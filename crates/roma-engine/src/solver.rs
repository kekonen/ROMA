use roma_core::{ExecutionContext, Result, SharedContext, TaskNode, NodeType, Event, EventType};
use roma_config::RomaConfig;
use roma_agents::{AgentFactory, Agents};
use roma_storage::{FileStorage, ExecutionStorage};
use std::sync::Arc;
use tracing::{info, warn, debug};

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

        // Clone data needed by the closure
        let context_for_closure = Arc::new(context.clone());
        let storage_for_closure = Arc::new(storage.clone());
        let subgraph_clone = subgraph.clone();
        let depth_clone = depth;
        let agents = self.agents.clone();
        let config = self.config.clone();

        self.event_loop
            .execute_with_dependencies(&subgraph, move |subtask| {
                let context = context_for_closure.clone();
                let storage = storage_for_closure.clone();
                let subgraph = subgraph_clone.clone();
                let agents = agents.clone();
                let config = config.clone();

                async move {
                    // Execute task (either atomic or compound)
                    let task = subgraph.get_task(&subtask.task_id)?;

                    if task.node_type == NodeType::Execute {
                        // Execute atomic task
                        let exec_storage = storage.as_ref();
                        execute_atomic_task_impl(&task, &context, exec_storage, &agents).await
                    } else {
                        // For compound tasks, create a mini-solver to handle recursion
                        // Since we can't use &self in the closure, we'll decompose inline
                        decompose_and_execute_inline(
                            &subtask.task_id,
                            &subgraph,
                            depth_clone + 1,
                            &context,
                            storage.as_ref(),
                            &agents,
                            &config,
                        ).await
                    }
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
                Vec::new(),
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

// Helper function for executing atomic tasks from closures
async fn execute_atomic_task_impl(
    task: &TaskNode,
    context: &SharedContext,
    storage: &ExecutionStorage,
    agents: &Agents,
) -> Result<TaskNode> {
    let mut task = task.clone();
    debug!("Executing atomic task {}", task.task_id);

    task.start_execution();

    let result = agents
        .executor
        .execute_task(
            &task.goal,
            Some(&context.execution_context().generate_agent_context(&task.goal)),
            Vec::new(),
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

// Helper function for decomposing and executing compound tasks recursively
async fn decompose_and_execute_inline(
    task_id: &str,
    dag: &TaskDag,
    depth: usize,
    context: &SharedContext,
    storage: &ExecutionStorage,
    agents: &Agents,
    config: &RomaConfig,
) -> Result<TaskNode> {
    let task = dag.get_task(task_id)?;

    // If we've hit max depth, force execute as atomic
    if depth >= config.runtime.max_depth {
        warn!("Max depth reached for task {}, force executing", task_id);
        return execute_atomic_task_impl(&task, context, storage, agents).await;
    }

    debug!("Planning compound task {}", task_id);

    let planning_result = agents
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

    // Create subgraph and execute subtasks
    let subgraph = TaskDag::new();
    let mut task_id_map = std::collections::HashMap::new();

    for subtask in &planning_result.subtasks {
        let task_node = subtask.clone().into_task_node(depth + 1, config.runtime.max_depth);
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

    // Execute subtasks sequentially (simplified, no parallel execution in nested calls)
    for subtask_meta in &planning_result.subtasks {
        if let Some(subtask_id) = task_id_map.get(&subtask_meta.task_id) {
            let subtask = subgraph.get_task(subtask_id)?;
            let completed = if subtask.node_type == NodeType::Execute {
                execute_atomic_task_impl(&subtask, context, storage, agents).await?
            } else {
                // Recursive call for nested compound tasks (boxed to avoid infinite size)
                Box::pin(decompose_and_execute_inline(
                    subtask_id,
                    &subgraph,
                    depth + 1,
                    context,
                    storage,
                    agents,
                    config,
                )).await?
            };

            // Update the subtask in the subgraph
            subgraph.update_task(subtask_id, |t| {
                t.result = completed.result.clone();
                t.status = completed.status;
            })?;
        }
    }

    // Aggregate results
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

    let aggregation_result = agents
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

    let mut final_task = task.clone();
    final_task.complete_execution(aggregation_result.synthesized_output.clone());

    // Verify result
    debug!("Verifying result for task {}", task_id);

    let verification_result = agents
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

    Ok(final_task)
}
