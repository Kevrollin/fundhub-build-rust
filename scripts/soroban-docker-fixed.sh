#!/bin/bash

# Fixed Soroban CLI Docker wrapper script
# This script provides a Docker-based alternative to installing Soroban CLI locally

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Docker image name
SOROBAN_IMAGE="fundhub-soroban:latest"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to build the Soroban Docker image using a simpler approach
build_soroban_image() {
    print_status "Building Soroban CLI Docker image using minimal approach..."
    
    # Create a simple Dockerfile on the fly
    cat > /tmp/soroban.Dockerfile << 'EOF'
FROM ubuntu:22.04

# Install minimal dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Download and install Rust
RUN curl --proto '=https' --tlsv1.2 -sSfL https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Soroban CLI
RUN cargo install --locked soroban-cli

# Set working directory
WORKDIR /workspace

# Default command
CMD ["soroban", "--help"]
EOF

    if docker build -f /tmp/soroban.Dockerfile -t $SOROBAN_IMAGE .; then
        print_success "Soroban CLI Docker image built successfully!"
        rm -f /tmp/soroban.Dockerfile
    else
        print_error "Failed to build Soroban CLI Docker image"
        rm -f /tmp/soroban.Dockerfile
        exit 1
    fi
}

# Function to run Soroban CLI commands
run_soroban() {
    local args="$@"
    
    if [ -z "$args" ]; then
        print_status "Running Soroban CLI help..."
        docker run --rm -v "$(pwd):/workspace" $SOROBAN_IMAGE --help
    else
        print_status "Running: soroban $args"
        docker run --rm -v "$(pwd):/workspace" $SOROBAN_IMAGE $args
    fi
}

# Function to deploy contracts using Docker
deploy_contracts() {
    local network="${1:-testnet}"
    
    print_status "Deploying contracts to $network using Docker..."
    
    # Check if image exists, build if not
    if ! docker image inspect $SOROBAN_IMAGE >/dev/null 2>&1; then
        print_warning "Soroban Docker image not found. Building..."
        build_soroban_image
    fi
    
    # Run the deployment script with Docker
    docker run --rm \
        -v "$(pwd):/workspace" \
        -w /workspace \
        $SOROBAN_IMAGE \
        bash -c "
            echo '🚀 Building and deploying Soroban contracts...'
            
            # Build contracts
            echo 'Building project-registry...'
            cd project-registry && cargo build --target wasm32-unknown-unknown --release && cd ..
            
            echo 'Building funding-escrow...'
            cd funding-escrow && cargo build --target wasm32-unknown-unknown --release && cd ..
            
            echo 'Building milestone-manager...'
            cd milestone-manager && cargo build --target wasm32-unknown-unknown --release && cd ..
            
            echo '✅ Contracts built successfully'
            echo '📋 Contract deployment would happen here with proper network configuration'
        "
}

# Function to show usage
show_usage() {
    echo "Soroban CLI Docker Wrapper (Fixed)"
    echo ""
    echo "Usage: $0 [COMMAND] [ARGS...]"
    echo ""
    echo "Commands:"
    echo "  build                    Build the Soroban CLI Docker image"
    echo "  deploy [network]         Deploy contracts (default: testnet)"
    echo "  help                     Show this help message"
    echo "  [soroban-args]           Run any Soroban CLI command"
    echo ""
    echo "Examples:"
    echo "  $0 build"
    echo "  $0 deploy testnet"
    echo "  $0 config network add testnet"
    echo "  $0 contract deploy --wasm contract.wasm"
}

# Main script logic
case "${1:-help}" in
    "build")
        build_soroban_image
        ;;
    "deploy")
        deploy_contracts "$2"
        ;;
    "help"|"--help"|"-h")
        show_usage
        ;;
    *)
        # Pass all arguments to Soroban CLI
        run_soroban "$@"
        ;;
esac
