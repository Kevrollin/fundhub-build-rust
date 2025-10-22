#!/bin/bash

# Simple development script for auto-reloading the backend
# Similar to Vite's hot reload for frontend

echo "🚀 FundHub Backend Development Server"
echo "🔄 Auto-reloading on code changes..."
echo ""

# Set environment variables
export RUST_LOG=debug
export DATABASE_URL="postgresql://dev_mk:Kevdev@2025@localhost:5432/fundhub2"

# Use cargo-watch with better options
cargo watch \
    --watch src/ \
    --watch Cargo.toml \
    --clear \
    --notify \
    --delay 0.5 \
    -- \
    run --bin fundhub
