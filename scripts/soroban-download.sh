#!/bin/bash

# Download Soroban CLI binary directly (lightweight approach)
# This avoids Docker build issues and gives you Soroban CLI quickly

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

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if soroban is already installed
if command -v soroban >/dev/null 2>&1; then
    print_success "Soroban CLI is already installed!"
    soroban --version
    exit 0
fi

print_status "Installing Soroban CLI via cargo (lightweight approach)..."

# Install Rust if not present
if ! command -v cargo >/dev/null 2>&1; then
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Install Soroban CLI with minimal features
print_status "Installing Soroban CLI (this may take a few minutes)..."
cargo install --locked soroban-cli

print_success "Soroban CLI installed successfully!"

# Test installation
soroban --version

echo ""
echo "🎉 Soroban CLI is ready to use!"
echo ""
echo "💡 Next steps:"
echo "   1. Configure network: soroban config network add testnet --rpc-url https://soroban-rpc.testnet.stellar.org --network-passphrase \"Test SDF Network ; September 2015\""
echo "   2. Create identity: soroban config identity generate fundhub-admin"
echo "   3. Fund identity: soroban config identity fund fundhub-admin --network testnet"
echo "   4. Deploy contracts: ./contracts/deploy.sh testnet"
