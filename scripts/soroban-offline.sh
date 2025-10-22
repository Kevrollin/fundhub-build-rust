#!/bin/bash

# Offline Soroban CLI approach - uses pre-built binaries or minimal setup
# This avoids Docker network issues entirely

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
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

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if soroban is already installed
if command -v soroban >/dev/null 2>&1; then
    print_success "Soroban CLI is already installed!"
    soroban --version
    exit 0
fi

print_status "Installing Soroban CLI using offline approach..."

# Method 1: Try to download pre-built binary
print_status "Attempting to download pre-built Soroban CLI binary..."

# Create a temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Try to download pre-built binary
if curl -L https://github.com/stellar/soroban-tools/releases/latest/download/soroban-cli-linux-x86_64.tar.gz -o soroban.tar.gz 2>/dev/null; then
    print_status "Downloaded pre-built binary, extracting..."
    tar -xzf soroban.tar.gz
    if [ -f soroban ]; then
        sudo mv soroban /usr/local/bin/
        sudo chmod +x /usr/local/bin/soroban
        print_success "Soroban CLI installed from pre-built binary!"
        soroban --version
        cd - >/dev/null
        rm -rf "$TEMP_DIR"
        exit 0
    fi
fi

print_warning "Pre-built binary not available, trying alternative approach..."

# Method 2: Use existing Rust installation or install minimal Rust
if ! command -v cargo >/dev/null 2>&1; then
    print_status "Installing minimal Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    source ~/.cargo/env
fi

# Install Soroban CLI with minimal features
print_status "Installing Soroban CLI via cargo (minimal approach)..."
cargo install --locked soroban-cli --no-default-features

print_success "Soroban CLI installed successfully!"

# Test installation
soroban --version

# Cleanup
cd - >/dev/null
rm -rf "$TEMP_DIR"

echo ""
echo "🎉 Soroban CLI is ready to use!"
echo ""
echo "💡 Next steps:"
echo "   1. Configure network: soroban config network add testnet --rpc-url https://soroban-rpc.testnet.stellar.org --network-passphrase \"Test SDF Network ; September 2015\""
echo "   2. Create identity: soroban config identity generate fundhub-admin"
echo "   3. Fund identity: soroban config identity fund fundhub-admin --network testnet"
echo "   4. Deploy contracts: ./contracts/deploy.sh testnet"
