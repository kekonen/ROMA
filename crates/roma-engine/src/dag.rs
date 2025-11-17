use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::{Direction, algo};
use roma_core::{Result, RomaError, TaskNode, TaskStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Clone)]
pub struct TaskDag {
    inner: Arc<RwLock<TaskDagInner>>,
}

struct TaskDagInner {
    graph: DiGraph<TaskNode, ()>,
    task_indices: HashMap<String, NodeIndex>,
    subgraphs: HashMap<String, TaskDag>,
}

impl TaskDag {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TaskDagInner {
                graph: DiGraph::new(),
                task_indices: HashMap::new(),
                subgraphs: HashMap::new(),
            })),
        }
    }

    pub fn add_task(&self, task: TaskNode) -> Result<String> {
        let mut inner = self.inner.write();
        let task_id = task.task_id.clone();

        if inner.task_indices.contains_key(&task_id) {
            return Err(RomaError::DagError(format!(
                "Task {} already exists",
                task_id
            )));
        }

        let index = inner.graph.add_node(task);
        inner.task_indices.insert(task_id.clone(), index);

        Ok(task_id)
    }

    pub fn add_dependency(&self, from_task_id: &str, to_task_id: &str) -> Result<()> {
        let mut inner = self.inner.write();

        let from_index = *inner
            .task_indices
            .get(from_task_id)
            .ok_or_else(|| RomaError::TaskNotFound(from_task_id.to_string()))?;

        let to_index = *inner
            .task_indices
            .get(to_task_id)
            .ok_or_else(|| RomaError::TaskNotFound(to_task_id.to_string()))?;

        inner.graph.add_edge(from_index, to_index, ());

        if algo::is_cyclic_directed(&inner.graph) {
            let edge = inner.graph.find_edge(from_index, to_index).unwrap();
            inner.graph.remove_edge(edge);
            return Err(RomaError::CycleDetected);
        }

        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Result<TaskNode> {
        let inner = self.inner.read();
        let index = inner
            .task_indices
            .get(task_id)
            .ok_or_else(|| RomaError::TaskNotFound(task_id.to_string()))?;

        Ok(inner.graph[*index].clone())
    }

    pub fn update_task<F>(&self, task_id: &str, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut TaskNode),
    {
        let mut inner = self.inner.write();
        let index = *inner
            .task_indices
            .get(task_id)
            .ok_or_else(|| RomaError::TaskNotFound(task_id.to_string()))?;

        if let Some(node) = inner.graph.node_weight_mut(index) {
            update_fn(node);
        }

        Ok(())
    }

    pub fn get_ready_tasks(&self) -> Vec<TaskNode> {
        let inner = self.inner.read();
        let mut ready = Vec::new();

        for node_index in inner.graph.node_indices() {
            let task = &inner.graph[node_index];

            if task.status != TaskStatus::Pending && task.status != TaskStatus::Ready {
                continue;
            }

            let all_deps_completed = inner
                .graph
                .edges_directed(node_index, Direction::Incoming)
                .all(|edge| {
                    let dep_task = &inner.graph[edge.source()];
                    dep_task.is_completed()
                });

            if all_deps_completed && task.status == TaskStatus::Pending {
                ready.push(task.clone());
            } else if task.status == TaskStatus::Ready {
                ready.push(task.clone());
            }
        }

        ready
    }

    pub fn mark_ready(&self, task_id: &str) -> Result<()> {
        self.update_task(task_id, |task| {
            task.status = TaskStatus::Ready;
        })
    }

    pub fn get_topological_order(&self) -> Result<Vec<String>> {
        let inner = self.inner.read();

        let sorted = algo::toposort(&inner.graph, None)
            .map_err(|_| RomaError::CycleDetected)?;

        Ok(sorted
            .iter()
            .map(|&idx| inner.graph[idx].task_id.clone())
            .collect())
    }

    pub fn get_dependencies(&self, task_id: &str) -> Result<Vec<String>> {
        let inner = self.inner.read();
        let index = inner
            .task_indices
            .get(task_id)
            .ok_or_else(|| RomaError::TaskNotFound(task_id.to_string()))?;

        let deps = inner
            .graph
            .edges_directed(*index, Direction::Incoming)
            .map(|edge| inner.graph[edge.source()].task_id.clone())
            .collect();

        Ok(deps)
    }

    pub fn get_dependents(&self, task_id: &str) -> Result<Vec<String>> {
        let inner = self.inner.read();
        let index = inner
            .task_indices
            .get(task_id)
            .ok_or_else(|| RomaError::TaskNotFound(task_id.to_string()))?;

        let dependents = inner
            .graph
            .edges_directed(*index, Direction::Outgoing)
            .map(|edge| inner.graph[edge.target()].task_id.clone())
            .collect();

        Ok(dependents)
    }

    pub fn all_tasks_completed(&self) -> bool {
        let inner = self.inner.read();
        inner.graph.node_weights().all(|task| task.is_completed())
    }

    pub fn has_failed_tasks(&self) -> bool {
        let inner = self.inner.read();
        inner.graph.node_weights().any(|task| task.is_failed())
    }

    pub fn get_all_tasks(&self) -> Vec<TaskNode> {
        let inner = self.inner.read();
        inner.graph.node_weights().cloned().collect()
    }

    pub fn task_count(&self) -> usize {
        let inner = self.inner.read();
        inner.graph.node_count()
    }

    pub fn add_subgraph(&self, subgraph_id: String, subgraph: TaskDag) {
        let mut inner = self.inner.write();
        inner.subgraphs.insert(subgraph_id, subgraph);
    }

    pub fn get_subgraph(&self, subgraph_id: &str) -> Option<TaskDag> {
        let inner = self.inner.read();
        inner.subgraphs.get(subgraph_id).cloned()
    }

    pub fn to_json(&self) -> Result<serde_json::Value> {
        let inner = self.inner.read();
        let tasks: Vec<&TaskNode> = inner.graph.node_weights().collect();

        let edges: Vec<(String, String)> = inner
            .graph
            .edge_references()
            .map(|edge| {
                let from = inner.graph[edge.source()].task_id.clone();
                let to = inner.graph[edge.target()].task_id.clone();
                (from, to)
            })
            .collect();

        Ok(serde_json::json!({
            "tasks": tasks,
            "edges": edges,
        }))
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self> {
        let dag = Self::new();

        if let Some(tasks_json) = json["tasks"].as_array() {
            for task_json in tasks_json {
                let task: TaskNode = serde_json::from_value(task_json.clone())
                    .map_err(|e| RomaError::SerializationError(e.to_string()))?;
                dag.add_task(task)?;
            }
        }

        if let Some(edges_json) = json["edges"].as_array() {
            for edge_json in edges_json {
                if let (Some(from), Some(to)) = (
                    edge_json[0].as_str(),
                    edge_json[1].as_str(),
                ) {
                    dag.add_dependency(from, to)?;
                }
            }
        }

        Ok(dag)
    }
}

impl Default for TaskDag {
    fn default() -> Self {
        Self::new()
    }
}
