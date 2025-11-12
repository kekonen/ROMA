use roma_core::{Result, RomaError, TaskNode, TaskStatus};
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinHandle;
use futures::future::join_all;
use dashmap::DashMap;

#[derive(Clone)]
struct PrioritizedTask {
    task: TaskNode,
    priority: usize,
}

impl Eq for PrioritizedTask {}

impl PartialEq for PrioritizedTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority).reverse()
    }
}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct EventLoopScheduler {
    max_concurrent: usize,
}

impl EventLoopScheduler {
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }

    pub async fn execute<F, Fut>(
        &self,
        tasks: Vec<TaskNode>,
        executor_fn: F,
    ) -> Result<Vec<TaskNode>>
    where
        F: Fn(TaskNode) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<TaskNode>> + Send + 'static,
    {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let results = Arc::new(DashMap::new());
        let executor_fn = Arc::new(executor_fn);

        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        for task in tasks {
            let sem = semaphore.clone();
            let results_map = results.clone();
            let executor = executor_fn.clone();
            let task_id = task.task_id.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                match executor(task.clone()).await {
                    Ok(completed_task) => {
                        results_map.insert(task_id, Ok(completed_task));
                    }
                    Err(e) => {
                        let mut failed_task = task.clone();
                        failed_task.fail_execution(e.to_string());
                        results_map.insert(task_id, Err(e));
                    }
                }
            });

            handles.push(handle);
        }

        join_all(handles).await;

        let mut completed_tasks = Vec::new();
        for entry in results.iter() {
            match entry.value() {
                Ok(task) => completed_tasks.push(task.clone()),
                Err(_) => {}
            }
        }

        Ok(completed_tasks)
    }

    pub async fn execute_with_dependencies<F, Fut>(
        &self,
        dag: &super::TaskDag,
        executor_fn: F,
    ) -> Result<()>
    where
        F: Fn(TaskNode) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<TaskNode>> + Send + 'static,
    {
        loop {
            let ready_tasks = dag.get_ready_tasks();

            if ready_tasks.is_empty() {
                if dag.all_tasks_completed() {
                    break;
                } else if dag.has_failed_tasks() {
                    return Err(RomaError::ExecutionError(
                        "One or more tasks failed".to_string(),
                    ));
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
            }

            for task in &ready_tasks {
                dag.update_task(&task.task_id, |t| {
                    t.status = TaskStatus::Executing;
                    t.start_execution();
                })?;
            }

            let completed = self.execute(ready_tasks, executor_fn.clone()).await?;

            for task in completed {
                dag.update_task(&task.task_id, |t| {
                    *t = task.clone();
                })?;

                let dependents = dag.get_dependents(&task.task_id)?;
                for dep_id in dependents {
                    let dep_task = dag.get_task(&dep_id)?;
                    let all_deps = dag.get_dependencies(&dep_id)?;

                    let all_completed = all_deps.iter().all(|dep_id| {
                        dag.get_task(dep_id).map(|t| t.is_completed()).unwrap_or(false)
                    });

                    if all_completed && dep_task.status == TaskStatus::Pending {
                        dag.mark_ready(&dep_id)?;
                    }
                }
            }
        }

        Ok(())
    }
}
