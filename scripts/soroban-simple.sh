#!/bin/bash

# Simple Soroban CLI Docker wrapper
# This script provides Soroban CLI via Docker without heavy setup

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Build Soroban CLI image if it doesn't exist
if ! docker image inspect fundhub-soroban:latest >/dev/null 2>&1; then
    print_status "Building Soroban CLI Docker image (this may take a few minutes)..."
    docker build -f docker/soroban.Dockerfile -t fundhub-soroban:latest .
    print_success "Soroban CLI Docker image built successfully!"
else
    print_status "Using existing Soroban CLI Docker image"
fi

# Run Soroban CLI with all arguments passed through
print_status "Running: soroban $@"
docker run --rm \
    -v "$(pwd):/workspace" \
    -w /workspace \
    fundhub-soroban:latest \
    "$@"
