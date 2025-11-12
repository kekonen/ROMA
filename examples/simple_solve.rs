use roma_config::ConfigManager;
use roma_engine::RecursiveSolver;
use roma_storage::FileStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("roma=debug".parse()?))
        .init();

    // Load configuration
    let config = ConfigManager::with_profile("general")?.into_config();

    // Initialize storage
    let storage = FileStorage::new(config.storage.base_path.clone());
    storage.init().await?;

    // Create solver
    let solver = RecursiveSolver::new(config, storage);

    // Solve a simple task
    let task = "Calculate 42 * 137 and then write the result to a file named result.txt";

    println!("Solving task: {}", task);

    match solver.solve(task.to_string()).await {
        Ok(result) => {
            println!("\n=== Task Completed Successfully ===");
            println!("Task ID: {}", result.task_id);
            if let Some(output) = result.result {
                println!("Result:\n{}", output);
            }
            if let Some(duration) = result.metrics.duration_ms {
                println!("Duration: {}ms", duration);
            }
        }
        Err(e) => {
            eprintln!("Task failed: {}", e);
        }
    }

    Ok(())
}
