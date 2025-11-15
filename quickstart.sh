#!/bin/bash

# ROMA Rust Quick Start Script
# This script helps you set up and run ROMA quickly

set -e

echo "🦀 ROMA Rust Quick Start"
echo "========================"
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo "   Install from: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust found: $(rustc --version)"

# Check for API keys
if [ -z "$ANTHROPIC_API_KEY" ] && [ -z "$OPENAI_API_KEY" ]; then
    echo ""
    echo "⚠️  No API keys found!"
    echo ""
    echo "Please set one of:"
    echo "  export ANTHROPIC_API_KEY='your-key-here'"
    echo "  export OPENAI_API_KEY='your-key-here'"
    echo ""
    read -p "Do you want to set an API key now? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo "Which provider?"
        echo "1) Anthropic (Claude)"
        echo "2) OpenAI (GPT)"
        read -p "Choice (1/2): " -n 1 -r provider
        echo ""

        if [[ $provider == "1" ]]; then
            read -p "Enter your Anthropic API key: " api_key
            export ANTHROPIC_API_KEY="$api_key"
            echo "export ANTHROPIC_API_KEY='$api_key'" >> ~/.bashrc
            echo "✅ Anthropic API key set and saved to ~/.bashrc"
        elif [[ $provider == "2" ]]; then
            read -p "Enter your OpenAI API key: " api_key
            export OPENAI_API_KEY="$api_key"
            echo "export OPENAI_API_KEY='$api_key'" >> ~/.bashrc
            echo "✅ OpenAI API key set and saved to ~/.bashrc"
        fi
    else
        exit 1
    fi
fi

if [ -n "$ANTHROPIC_API_KEY" ]; then
    echo "✅ Anthropic API key found"
fi

if [ -n "$OPENAI_API_KEY" ]; then
    echo "✅ OpenAI API key found"
fi

echo ""
echo "Building ROMA..."
echo ""

# Build the project
cargo build --release

echo ""
echo "✅ Build complete!"
echo ""
echo "════════════════════════════════════════"
echo "  ROMA is ready! Choose an option:"
echo "════════════════════════════════════════"
echo ""
echo "1) Run a simple test task"
echo "2) Start API server"
echo "3) Show configuration"
echo "4) Run custom task"
echo "5) Exit"
echo ""
read -p "Your choice (1-5): " -n 1 -r choice
echo ""

case $choice in
    1)
        echo ""
        echo "Running test task: 'Explain what 2+2 equals'"
        echo ""
        cargo run --release --bin roma -- solve "Explain what 2+2 equals"
        ;;
    2)
        echo ""
        echo "Starting API server on port 8080..."
        echo "API will be available at: http://localhost:8080"
        echo ""
        cargo run --release --bin roma -- server --port 8080
        ;;
    3)
        echo ""
        cargo run --release --bin roma -- config
        ;;
    4)
        echo ""
        read -p "Enter your task: " task
        echo ""
        cargo run --release --bin roma -- solve "$task"
        ;;
    5)
        echo "Goodbye!"
        exit 0
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

echo ""
echo "════════════════════════════════════════"
echo "  Quick Reference:"
echo "════════════════════════════════════════"
echo ""
echo "Solve a task:"
echo "  cargo run --bin roma -- solve 'Your task here'"
echo ""
echo "Start server:"
echo "  cargo run --bin roma -- server"
echo ""
echo "View config:"
echo "  cargo run --bin roma -- config"
echo ""
echo "Full documentation: See RUST_SETUP.md"
echo ""
