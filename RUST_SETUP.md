# 🦀 ROMA Rust Setup Guide

Complete guide to configure and run ROMA with Rust using rig-core 0.24.0.

## 📋 Prerequisites

- **Rust 1.91.0+** (Rust 2024 edition)
- **Cargo** (comes with Rust)
- **API Keys**: Anthropic or OpenAI API key

## ⚡ Quick Start

### 1. Set Up API Keys

```bash
# For Anthropic (Claude)
export ANTHROPIC_API_KEY="your-anthropic-api-key-here"

# OR for OpenAI
export OPENAI_API_KEY="your-openai-api-key-here"
```

Add to your shell profile for persistence:

```bash
# ~/.bashrc or ~/.zshrc
echo 'export ANTHROPIC_API_KEY="your-key-here"' >> ~/.bashrc
source ~/.bashrc
```

### 2. Build the Project

```bash
# Build all crates
cargo build --release

# Or just build in debug mode (faster compilation)
cargo build
```

### 3. Run ROMA CLI

```bash
# Solve a task
cargo run --bin roma -- solve "Explain quantum computing"

# With custom settings
cargo run --bin roma -- solve "Write a factorial function" \
  --max-depth 5 \
  --timeout 120

# Using a specific profile
cargo run --bin roma -- --profile general solve "Your task here"
```

## 🔧 Configuration

### Configuration Files

ROMA uses YAML configuration files located in `config/`:

```
config/
├── defaults/
│   └── config.yaml         # Base configuration
└── profiles/
    ├── general.yaml        # General-purpose profile
    ├── test.yaml          # Testing profile
    └── crypto_agent.yaml  # Crypto-specific profile
```

### Configuration Hierarchy

1. **defaults/config.yaml** - Base configuration with environment variable substitution
2. **profiles/{name}.yaml** - Profile-specific overrides
3. **Command-line arguments** - Highest priority

### Key Configuration Options

#### `config/profiles/general.yaml`

```yaml
agents:
  atomizer:
    llm:
      provider: anthropic  # or "openai"
      model: claude-sonnet-4-5-20250514
      temperature: 0.7
      max_tokens: 4096

  planner:
    llm:
      provider: anthropic
      model: claude-sonnet-4-5-20250514
      temperature: 0.7
      max_tokens: 8192

  executor:
    llm:
      provider: anthropic
      model: claude-sonnet-4-5-20250514
      temperature: 0.0
      max_tokens: 8192
    toolkits:
      - artifact
      - file
      - calculator

runtime:
  max_depth: 6              # Maximum recursion depth
  timeout: 120              # Timeout in seconds
  max_concurrent_tasks: 4   # Parallel task execution

storage:
  base_path: /tmp/roma      # Where execution data is stored

resilience:
  retry:
    enabled: true
    strategy: Exponential
    max_attempts: 3
```

### Environment Variables

```bash
# LLM Provider Keys
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."

# Runtime Configuration
export ROMA_ENV="production"
export ROMA_MAX_DEPTH="6"
export ROMA_VERBOSE="true"
export ROMA_ENABLE_LOGGING="true"
export ROMA_LOG_LEVEL="INFO"

# Storage
export STORAGE_BASE_PATH="/tmp/roma"

# Optional: PostgreSQL (for persistent storage)
export POSTGRES_ENABLED="false"
export DATABASE_URL="postgresql://localhost/roma"

# Optional: Observability
export MLFLOW_ENABLED="false"
export MLFLOW_TRACKING_URI="http://127.0.0.1:5000"
```

## 🚀 Usage Examples

### CLI Commands

#### 1. Solve a Task

```bash
# Basic usage
cargo run --bin roma -- solve "Calculate 15 factorial"

# With verbose output
cargo run --bin roma -- --verbose solve "Explain recursion"

# With custom depth and timeout
cargo run --bin roma -- solve "Write a quicksort algorithm" \
  --max-depth 8 \
  --timeout 300
```

#### 2. Start API Server

```bash
# Start server on default port (8080)
cargo run --bin roma -- server

# Custom port
cargo run --bin roma -- server --port 3000
```

#### 3. View Configuration

```bash
# Show current configuration
cargo run --bin roma -- config

# Show specific profile
cargo run --bin roma -- config general
```

### API Usage

Once the server is running:

```bash
# Health check
curl http://localhost:8080/health

# Create execution
curl -X POST http://localhost:8080/api/executions \
  -H "Content-Type: application/json" \
  -d '{"goal": "Explain machine learning"}'

# Get execution status
curl http://localhost:8080/api/executions/{execution_id}

# Get DAG structure
curl http://localhost:8080/api/executions/{execution_id}/dag

# Get metrics
curl http://localhost:8080/api/metrics
```

### TUI (Terminal UI)

```bash
# Run the terminal UI
cargo run --bin roma-tui

# The TUI provides an interactive interface for:
# - Viewing running executions
# - Monitoring task DAGs
# - Inspecting logs and events
```

## 🏗️ Architecture Overview

### Crates Structure

```
crates/
├── roma-core/          # Core types and traits
├── roma-config/        # Configuration management
├── roma-storage/       # File and database storage
├── roma-agents/        # Agent implementations
│   ├── Atomizer       # Decides if task is atomic
│   ├── Planner        # Breaks down complex tasks
│   ├── Executor       # Executes atomic tasks
│   ├── Aggregator     # Combines subtask results
│   └── Verifier       # Validates results
├── roma-engine/        # DAG execution engine
├── roma-toolkit/       # Tool implementations
│   ├── File tools     # Read/write files
│   ├── Calculator     # Math operations
│   ├── Artifact tools # Artifact management
│   └── Docker tools   # Code execution
├── roma-resilience/    # Retry & checkpointing
├── roma-observability/ # Tracing & metrics
├── roma-api/          # REST API server
├── roma-cli/          # Command-line interface
└── roma-tui/          # Terminal UI
```

### Agent Flow

```
User Request
     ↓
[Atomizer] → Is atomic?
     ↓ No           ↓ Yes
[Planner]      [Executor]
     ↓               ↓
Subtasks → [Recursive Solve]
     ↓
[Aggregator]
     ↓
[Verifier]
     ↓
Result
```

## 🛠️ Available Toolkits

### Built-in Tools

1. **File Toolkit** (`file`)
   - `read_file`: Read file contents
   - `write_file`: Write to files
   - `list_files`: List files in directory

2. **Calculator Toolkit** (`calculator`)
   - `add`: Addition
   - `subtract`: Subtraction
   - `multiply`: Multiplication
   - `divide`: Division

3. **Artifact Toolkit** (`artifact`)
   - `create_artifact`: Store artifacts
   - `get_artifact`: Retrieve artifacts

4. **Docker Toolkit** (`docker`) *(optional)*
   - `run_python_code`: Execute Python in container
   - `run_command`: Run shell commands

### Configuring Toolkits

Edit `config/profiles/general.yaml`:

```yaml
agents:
  executor:
    toolkits:
      - artifact
      - file
      - calculator
      # - docker  # Enable if Docker is available
```

## 🔍 Debugging & Troubleshooting

### Enable Verbose Logging

```bash
# Via environment
export ROMA_VERBOSE="true"
export ROMA_LOG_LEVEL="DEBUG"

# Via CLI flag
cargo run --bin roma -- --verbose solve "Your task"
```

### Check Configuration

```bash
# View loaded configuration
cargo run --bin roma -- config

# Test with minimal example
cargo run --bin roma -- solve "Say hello"
```

### Common Issues

#### 1. API Key Not Set
```
Error: ANTHROPIC_API_KEY not set
```
**Solution:** Set the environment variable:
```bash
export ANTHROPIC_API_KEY="your-key-here"
```

#### 2. Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build
```

#### 3. Connection Timeouts
Increase timeout in config:
```yaml
runtime:
  timeout: 300  # 5 minutes
```

### View Execution Data

```bash
# Execution data is stored at:
ls -la /tmp/roma/

# Each execution creates:
# /tmp/roma/{execution_id}/
#   ├── tasks/        # Task information
#   ├── artifacts/    # Created artifacts
#   └── events.json   # Execution events
```

## 📊 Observability

### Tracing

ROMA uses OpenTelemetry for distributed tracing:

```yaml
observability:
  tracing:
    enabled: true
    endpoint: null  # Set to OTLP endpoint if using external collector
    service_name: roma
```

### Events

All execution events are tracked:
- Task creation
- Planning decisions
- Execution results
- Aggregation
- Verification

Access via:
```bash
# View events for an execution
cat /tmp/roma/{execution_id}/events.json
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p roma-agents
cargo test -p roma-engine

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_task_creation
```

## 📦 Building for Production

### Release Build

```bash
# Optimized release build
cargo build --release

# Binary location
./target/release/roma

# Install to system
cargo install --path crates/roma-cli
```

### Docker Deployment

Create `Dockerfile`:

```dockerfile
FROM rust:1.91 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/roma /usr/local/bin/
ENV ANTHROPIC_API_KEY=""
EXPOSE 8080
CMD ["roma", "server", "--port", "8080"]
```

Build and run:
```bash
docker build -t roma .
docker run -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY -p 8080:8080 roma
```

## 🔐 Security Notes

- **Never commit API keys** to version control
- Store keys in environment variables or secret management systems
- Use `.gitignore` for local config overrides
- Consider using Docker secrets in production

## 📚 Advanced Configuration

### Custom Profiles

Create `config/profiles/my-profile.yaml`:

```yaml
agents:
  executor:
    llm:
      model: claude-opus-4-20250514  # More powerful model
      temperature: 0.0
    toolkits:
      - artifact
      - file
      - calculator
      - docker

runtime:
  max_depth: 10
  max_concurrent_tasks: 8
```

Use it:
```bash
cargo run --bin roma -- --profile my-profile solve "Complex task"
```

### PostgreSQL Integration

For persistent storage:

```yaml
storage:
  postgres:
    enabled: true
    connection_url: postgresql://user:pass@localhost/roma
    pool_size: 10
```

### MLflow Integration

For experiment tracking:

```yaml
observability:
  mlflow:
    enabled: true
    tracking_uri: http://127.0.0.1:5000
    experiment_name: ROMA-Experiments
```

## 🎯 Performance Tuning

### Parallel Execution

```yaml
runtime:
  max_concurrent_tasks: 8  # Increase for more parallelism
```

### Caching

```yaml
runtime:
  cache:
    enabled: true
    enable_disk_cache: true
    enable_memory_cache: true
    memory_max_entries: 1000000
```

### Retry Strategy

```yaml
resilience:
  retry:
    enabled: true
    strategy: Exponential
    max_attempts: 5
    base_delay_ms: 1000
    max_delay_ms: 30000
```

## 📖 Additional Resources

- **README.md** - Main project documentation
- **config/examples/** - Example configurations
- **crates/*/src/lib.rs** - API documentation

## 💡 Tips

1. Start with the `general` profile for most tasks
2. Use `--verbose` flag to understand agent decisions
3. Monitor `/tmp/roma/` to see execution artifacts
4. Adjust `max_depth` based on task complexity
5. Use the API server for long-running tasks
6. Check logs in the configured log directory

---

**Need Help?** Check the issues on GitHub or join the Discord community!
