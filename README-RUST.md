# ROMA - Rust Implementation

This is a complete Rust rewrite of the ROMA (Recursive Open Meta-Agents) framework using `rig-core 0.24.0` and the Rust ecosystem.

## Architecture

ROMA is a hierarchical multi-agent task decomposition framework that implements a recursive plan-execute loop for solving complex tasks through intelligent decomposition.

### Core Components

1. **5-Agent System** (using rig-core)
   - **Atomizer**: Determines if a task is atomic or needs decomposition
   - **Planner**: Breaks non-atomic tasks into subtasks with dependencies
   - **Executor**: Executes atomic tasks using available tools
   - **Aggregator**: Synthesizes results from multiple subtasks
   - **Verifier**: Validates outputs satisfy the original goal

2. **Execution Engine**
   - **DAG System**: Directed acyclic graph for task dependencies (petgraph)
   - **Recursive Solver**: Orchestrates the entire execution flow
   - **Event Loop**: Parallel task execution with concurrency control

3. **Toolkit System**
   - **File Toolkit**: File operations within execution-scoped storage
   - **Calculator Toolkit**: Basic arithmetic operations
   - **Artifact Toolkit**: Task artifact management (code, documents, data)
   - **Docker Toolkit**: Sandboxed code execution using Docker containers
   - **MCP Toolkit**: Model Context Protocol integration via mcp-rig

4. **Storage Layer**
   - **FileStorage**: Execution-scoped file system with isolation
   - **PostgresStorage**: Async persistence for executions, traces, checkpoints
   - **Parquet Support**: Efficient storage for large data artifacts

5. **Resilience**
   - **Retry Policies**: Exponential backoff, fixed delay, jittered
   - **Circuit Breaker**: Automatic failure detection and recovery
   - **Checkpoint Manager**: Periodic snapshots for failure recovery

6. **Observability**
   - **OpenTelemetry**: Distributed tracing support
   - **Structured Logging**: tracing-subscriber with multiple outputs
   - **Metrics Collection**: Tool invocation tracking

7. **Interfaces**
   - **REST API**: Axum-based HTTP server
   - **CLI**: Clap-based command-line interface
   - **TUI**: Ratatui-based terminal UI for visualization

## Project Structure

```
ROMA/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── roma-core/          # Core data models and error types
│   ├── roma-config/        # Configuration management
│   ├── roma-agents/        # 5-agent system with rig-core
│   ├── roma-engine/        # Execution engine (DAG, solver, event loop)
│   ├── roma-toolkit/       # Tool system and implementations
│   ├── roma-storage/       # Storage layer (file, postgres, parquet)
│   ├── roma-resilience/    # Retry, circuit breaker, checkpoints
│   ├── roma-observability/ # Tracing and logging
│   ├── roma-api/           # REST API with axum
│   ├── roma-cli/           # CLI application
│   └── roma-tui/           # Terminal UI
└── config/
    └── profiles/
        ├── general.yaml    # General configuration
        └── test.yaml       # Test configuration
```

## Installation

### Prerequisites

- Rust 1.75+ (edition 2021)
- Docker (for docker toolkit)
- PostgreSQL (optional, for postgres storage)

### Build

```bash
cargo build --release
```

The binary will be available at `target/release/roma`.

## Configuration

Configuration files are YAML-based and can be organized by profiles:

```yaml
agents:
  atomizer:
    llm:
      provider: openai  # or anthropic
      model: gpt-4
      temperature: 0.7
      max_tokens: 4096
    prediction_strategy: ChainOfThought
    toolkits: []
  # ... other agents

runtime:
  max_depth: 6
  timeout: 120
  verbose: true
  max_concurrent_tasks: 4

resilience:
  retry:
    enabled: true
    strategy: Exponential
    max_attempts: 3
  # ... other resilience config

storage:
  base_path: /tmp/roma
  postgres:
    enabled: false
    connection_url: ""
```

## Usage

### CLI

```bash
# Solve a task
roma solve "Create a Python script to analyze weather data"

# With custom configuration
roma --config my-config.yaml solve "Your task here"

# With specific profile
roma --profile test solve "Your task here"

# Start API server
roma server --port 8080

# Show configuration
roma config
```

### Environment Variables

Required environment variables:

```bash
# For OpenAI provider
export OPENAI_API_KEY=your_api_key

# For Anthropic provider
export ANTHROPIC_API_KEY=your_api_key

# Optional: PostgreSQL connection
export ROMA__STORAGE__POSTGRES__CONNECTION_URL=postgresql://user:pass@localhost/roma
```

### Programmatic Usage

```rust
use roma_config::ConfigManager;
use roma_engine::RecursiveSolver;
use roma_storage::FileStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigManager::with_profile("general")?.into_config();
    let storage = FileStorage::new(config.storage.base_path.clone());
    storage.init().await?;

    let solver = RecursiveSolver::new(config, storage);
    let result = solver.solve("Your task here".to_string()).await?;

    println!("Result: {:?}", result.result);
    Ok(())
}
```

## Features

### Task Types (MECE Framework)

- **RETRIEVE**: External data acquisition (API calls, web searches)
- **WRITE**: Content generation and synthesis
- **THINK**: Analysis, reasoning, decision making
- **CODE_INTERPRET**: Code execution and data processing
- **IMAGE_GENERATION**: Visual content creation

### Prediction Strategies

- **ChainOfThought**: Step-by-step reasoning
- **ReAct**: Reasoning and acting with tools
- **CodeAct**: Code-based action execution
- **BestOfN**: Multiple attempts with best selection
- **Direct**: Direct completion without special prompting

### Toolkits

All toolkits implement the `Toolkit` trait and integrate seamlessly with rig-core's tool system:

```rust
#[async_trait]
pub trait Toolkit: Send + Sync {
    fn name(&self) -> &str;
    fn tools(&self) -> Vec<Arc<dyn rig_core::tool::Tool>>;
    async fn setup(&mut self) -> Result<()>;
    async fn cleanup(&mut self) -> Result<()>;
}
```

### MCP Integration

ROMA supports the Model Context Protocol via `mcp-rig`:

```rust
let mcp_toolkit = McpToolkit::new_stdio(
    "filesystem".to_string(),
    "npx",
    vec!["@modelcontextprotocol/server-filesystem".to_string()],
).await?;
```

## Key Differences from Python Version

1. **Type Safety**: Full compile-time type checking with Rust's type system
2. **Performance**: Native performance with zero-cost abstractions
3. **Concurrency**: True parallelism with tokio and async/await
4. **Memory Safety**: No garbage collection, guaranteed memory safety
5. **LLM Integration**: Direct rig-core integration instead of DSPy
6. **Error Handling**: Result types instead of exceptions
7. **Tooling**: Cargo-based build system and ecosystem

## Development

### Running Tests

```bash
cargo test
```

### Running with Verbose Logging

```bash
RUST_LOG=debug cargo run -- solve "Your task"
```

### Adding New Toolkits

1. Implement the `Toolkit` trait
2. Create tools implementing `rig_core::tool::Tool`
3. Register in `ToolkitManager`

Example:

```rust
use async_trait::async_trait;
use rig_core::tool::Tool;

#[derive(Clone)]
struct MyTool;

#[async_trait]
impl Tool for MyTool {
    const NAME: &'static str = "my_tool";
    type Error = String;
    type Args = MyArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> rig_core::tool::ToolDefinition {
        // Define tool schema
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Implement tool logic
    }
}
```

## Dependencies

### Core
- `rig-core = "0.24.0"` - LLM agent framework
- `tokio = "1.42"` - Async runtime
- `serde = "1.0"` - Serialization

### Storage
- `sqlx = "0.8"` - PostgreSQL async driver
- `parquet = "54.0"` - Parquet file format
- `petgraph = "0.6"` - Graph data structures

### Observability
- `tracing = "0.1"` - Structured logging
- `opentelemetry = "0.27"` - Distributed tracing

### API & CLI
- `axum = "0.7"` - HTTP framework
- `clap = "4.5"` - CLI parser
- `ratatui = "0.29"` - Terminal UI

### MCP
- `mcp-rig = "0.1.0"` - MCP integration for rig
- `mcp-client = "0.4"` - MCP client library

## License

MIT

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. Documentation is updated
