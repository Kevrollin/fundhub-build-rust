#!/bin/bash

# Development script for auto-reloading the backend
# This script watches for changes in the Rust code and automatically rebuilds and restarts the server

echo "🚀 Starting FundHub Backend Development Server with Auto-Reload"
echo "📁 Watching for changes in: src/"
echo "🔄 Auto-restarting on code changes..."
echo ""

# Set environment variables for development
export RUST_LOG=debug
export DATABASE_URL="postgresql://dev_mk:Kevdev@2025@localhost:5432/fundhub2"

# Use cargo-watch to automatically rebuild and restart on file changes
cargo watch -x "run --bin fundhub" \
    --watch src/ \
    --watch Cargo.toml \
    --watch Cargo.lock \
    --clear \
    --notify \
    --delay 0.5
