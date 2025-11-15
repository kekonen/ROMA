# 🚀 ROMA Rust Quick Start

Get ROMA up and running in 5 minutes!

## Prerequisites

- Rust 1.91+ installed ([Install Rust](https://rustup.rs/))
- An API key from Anthropic or OpenAI

## 1️⃣ Set Your API Key

```bash
# For Anthropic (recommended)
export ANTHROPIC_API_KEY="your-anthropic-api-key"

# OR for OpenAI
export OPENAI_API_KEY="your-openai-api-key"
```

## 2️⃣ Build & Run

### Option A: Use the Quick Start Script

```bash
./quickstart.sh
```

This interactive script will:
- Check your environment
- Build the project
- Help you run your first task

### Option B: Manual Commands

```bash
# Build
cargo build --release

# Run a simple task
cargo run --bin roma -- solve "What is 2+2?"

# Start the API server
cargo run --bin roma -- server
```

## 3️⃣ Try Some Examples

### Simple Math
```bash
cargo run --bin roma -- solve "Calculate factorial of 10"
```

### Code Generation
```bash
cargo run --bin roma -- solve "Write a Python function to reverse a string"
```

### Research
```bash
cargo run --bin roma -- solve "Explain quantum computing in simple terms"
```

## 4️⃣ Use the API

Start the server:
```bash
cargo run --bin roma -- server --port 8080
```

Make requests:
```bash
# Create execution
curl -X POST http://localhost:8080/api/executions \
  -H "Content-Type: application/json" \
  -d '{"goal": "Explain machine learning"}'

# Health check
curl http://localhost:8080/health
```

## 📚 Next Steps

- Read **RUST_SETUP.md** for detailed configuration
- Check **config/profiles/** for configuration examples
- Explore **config/examples/** for advanced setups

## 💡 Common Commands

```bash
# Solve with custom depth
cargo run --bin roma -- solve "Your task" --max-depth 8

# Use different profile
cargo run --bin roma -- --profile test solve "Your task"

# View configuration
cargo run --bin roma -- config

# Run with verbose output
cargo run --bin roma -- --verbose solve "Your task"
```

## 🔍 Troubleshooting

### "API key not set"
```bash
export ANTHROPIC_API_KEY="your-key"
```

### Build errors
```bash
cargo clean
cargo build
```

### View execution data
```bash
ls /tmp/roma/
```

## 🎯 What Next?

1. **Customize**: Edit `config/profiles/general.yaml`
2. **Integrate**: Use the REST API in your applications
3. **Extend**: Add custom tools in `crates/roma-toolkit/`
4. **Monitor**: Enable observability features

## 📖 Documentation

- `RUST_SETUP.md` - Complete setup guide
- `README.md` - Project overview
- `config/` - Configuration examples

**Happy coding! 🦀**
