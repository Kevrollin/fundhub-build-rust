#!/bin/bash

# Simple contract deployment using Docker-based Soroban CLI
# Usage: ./scripts/deploy-contracts-simple.sh [testnet|mainnet]

set -e

NETWORK=${1:-testnet}

echo "🚀 Deploying FundHub Smart Contracts to $NETWORK (Docker-based)"

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

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
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

# Set network-specific configuration
if [ "$NETWORK" = "mainnet" ]; then
    echo "⚠️  WARNING: Deploying to MAINNET"
    RPC_URL="https://horizon.stellar.org"
    NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
    SOROBAN_RPC_URL="https://soroban-rpc.mainnet.stellar.org"
else
    echo "📝 Deploying to TESTNET"
    RPC_URL="https://horizon-testnet.stellar.org"
    NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
    SOROBAN_RPC_URL="https://soroban-rpc.testnet.stellar.org"
fi

print_status "Building contracts using Docker..."

# Build all contracts in one Docker run
docker run --rm \
    -v "$(pwd):/workspace" \
    -w /workspace \
    fundhub-soroban:latest \
    bash -c "
        echo 'Building project-registry...'
        cd project-registry && cargo build --target wasm32-unknown-unknown --release && cd ..
        
        echo 'Building funding-escrow...'
        cd funding-escrow && cargo build --target wasm32-unknown-unknown --release && cd ..
        
        echo 'Building milestone-manager...'
        cd milestone-manager && cargo build --target wasm32-unknown-unknown --release && cd ..
        
        echo 'Optimizing WASM files...'
        soroban contract optimize --wasm project-registry/target/wasm32-unknown-unknown/release/project_registry.wasm --wasm-out project-registry/target/wasm32-unknown-unknown/release/project_registry_optimized.wasm
        soroban contract optimize --wasm funding-escrow/target/wasm32-unknown-unknown/release/funding_escrow.wasm --wasm-out funding-escrow/target/wasm32-unknown-unknown/release/funding_escrow_optimized.wasm
        soroban contract optimize --wasm milestone-manager/target/wasm32-unknown-unknown/release/milestone_manager.wasm --wasm-out milestone-manager/target/wasm32-unknown-unknown/release/milestone_manager_optimized.wasm
        
        echo '✅ Contracts built and optimized successfully'
    "

print_success "Contract build completed!"

# Show what would be deployed
echo ""
echo "📋 Contract Deployment Summary:"
echo "   Network: $NETWORK"
echo "   Project Registry: [Ready for deployment]"
echo "   Funding Escrow: [Ready for deployment]"
echo "   Milestone Manager: [Ready for deployment]"
echo ""
echo "🔧 Next Steps:"
echo "   1. Configure Soroban CLI:"
echo "      ./scripts/soroban-simple.sh config network add $NETWORK --rpc-url $SOROBAN_RPC_URL --network-passphrase \"$NETWORK_PASSPHRASE\""
echo ""
echo "   2. Create a wallet:"
echo "      ./scripts/soroban-simple.sh config identity generate fundhub-admin"
echo "      ./scripts/soroban-simple.sh config identity fund fundhub-admin --network $NETWORK"
echo ""
echo "   3. Deploy contracts:"
echo "      ./scripts/soroban-simple.sh contract deploy --wasm project-registry/target/wasm32-unknown-unknown/release/project_registry_optimized.wasm --source-account fundhub-admin --network $NETWORK"
echo ""
echo "💡 All Soroban commands can be run with: ./scripts/soroban-simple.sh [command]"

print_success "Docker-based contract build completed successfully!"
