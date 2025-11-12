use roma_core::{TaskNode, TaskStatus, SubTask, TaskType};

#[test]
fn test_task_node_creation() {
    let mut task = TaskNode::new("Calculate factorial of 10".to_string(), 0, 6);

    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.depth, 0);
    assert_eq!(task.max_depth, 6);

    task.start_execution();
    assert_eq!(task.status, TaskStatus::Executing);
    assert!(task.metrics.start_time.is_some());

    task.complete_execution("Result: 3628800".to_string());
    assert_eq!(task.status, TaskStatus::Completed);
    assert!(task.result.is_some());
    assert!(task.metrics.duration_ms.is_some());

    println!("✅ TaskNode state management works correctly");
}

#[test]
fn test_subtask_creation() {
    let subtask = SubTask::new("Retrieve data from API".to_string(), TaskType::Retrieve)
        .with_dependencies(vec!["task_1".to_string()]);

    assert_eq!(subtask.task_type, TaskType::Retrieve);
    assert_eq!(subtask.dependencies.len(), 1);

    let task_node = subtask.into_task_node(1, 6);
    assert_eq!(task_node.depth, 1);
    assert_eq!(task_node.max_depth, 6);

    println!("✅ SubTask creation and conversion works correctly");
}

#[test]
fn test_task_type_enum() {
    assert_eq!(TaskType::Retrieve.as_str(), "RETRIEVE");
    assert_eq!(TaskType::Write.as_str(), "WRITE");
    assert_eq!(TaskType::Think.as_str(), "THINK");
    assert_eq!(TaskType::CodeInterpret.as_str(), "CODE_INTERPRET");
    assert_eq!(TaskType::ImageGeneration.as_str(), "IMAGE_GENERATION");

    println!("✅ TaskType enum works correctly");
}
