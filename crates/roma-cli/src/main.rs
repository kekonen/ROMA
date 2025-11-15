use clap::{Parser, Subcommand};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use roma_config::ConfigManager;
use roma_engine::RecursiveSolver;
use roma_observability;
use roma_storage::FileStorage;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "roma")]
#[command(about = "ROMA - Recursive Open Meta-Agents", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration profile to use
    #[arg(short, long, default_value = "general")]
    profile: String,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Solve a task using ROMA
    Solve {
        /// The task goal to accomplish
        goal: String,

        /// Maximum recursion depth
        #[arg(long)]
        max_depth: Option<usize>,

        /// Timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,
    },

    /// Start the API server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Show configuration
    Config {
        /// Show specific profile
        profile: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    let config = if let Some(config_path) = cli.config {
        ConfigManager::from_file(config_path)?
    } else {
        ConfigManager::with_profile(&cli.profile)?
    };

    let config = config.into_config();

    roma_observability::init_tracing(
        &config.observability.tracing.service_name,
        config.observability.tracing.endpoint.as_deref(),
    )?;

    match cli.command {
        Commands::Solve {
            goal,
            max_depth,
            timeout,
        } => {
            let mut config = config;

            if let Some(depth) = max_depth {
                config.runtime.max_depth = depth;
            }

            if let Some(t) = timeout {
                config.runtime.timeout = t;
            }

            println!("{}", style("ROMA - Recursive Open Meta-Agents").cyan().bold());
            println!("{}", style("=".repeat(50)).dim());
            println!();
            println!("{} {}", style("Goal:").green().bold(), goal);
            println!("{} {}", style("Max Depth:").green().bold(), config.runtime.max_depth);
            println!();

            let storage = FileStorage::new(config.storage.base_path.clone());
            storage.init().await?;

            let solver = RecursiveSolver::new(config, storage);

            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            pb.set_message("Solving task...");
            pb.enable_steady_tick(std::time::Duration::from_millis(100));

            match solver.solve(goal).await {
                Ok(result) => {
                    pb.finish_and_clear();

                    println!();
                    println!("{}", style("✓ Task Completed Successfully").green().bold());
                    println!("{}", style("=".repeat(50)).dim());
                    println!();

                    if let Some(output) = result.result {
                        println!("{}", style("Result:").cyan().bold());
                        println!("{}", output);
                    }

                    println!();
                    println!("{} {}", style("Task ID:").dim(), result.task_id);
                    if let Some(duration) = result.metrics.duration_ms {
                        println!("{} {}ms", style("Duration:").dim(), duration);
                    }
                }
                Err(e) => {
                    pb.finish_and_clear();

                    println!();
                    println!("{}", style("✗ Task Failed").red().bold());
                    println!("{}", style("=".repeat(50)).dim());
                    println!();
                    println!("{} {}", style("Error:").red(), e);
                }
            }

            roma_observability::shutdown_tracing();
        }

        Commands::Server { port } => {
            println!("{}", style("ROMA API Server").cyan().bold());
            println!("{}", style("=".repeat(50)).dim());
            println!();

            let storage = FileStorage::new(config.storage.base_path.clone());
            storage.init().await?;

            let server = roma_api::ApiServer::new(config, storage, port);
            server.run().await?;
        }

        Commands::Config { profile } => {
            let profile_name = profile.as_deref().unwrap_or(&cli.profile);

            println!("{}", style(format!("Configuration Profile: {}", profile_name)).cyan().bold());
            println!("{}", style("=".repeat(50)).dim());
            println!();

            let json = serde_json::to_string_pretty(&config)?;
            println!("{}", json);
        }
    }

    Ok(())
}
