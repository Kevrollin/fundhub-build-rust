#!/bin/bash

set -e

echo "🚀 Building and deploying Soroban contracts..."

# Check if soroban CLI is installed
if ! command -v soroban &> /dev/null; then
    echo "❌ Soroban CLI is not installed."
    echo ""
    echo "🔧 Quick installation options:"
    echo "   1. Lightweight install: ./scripts/soroban-download.sh"
    echo "   2. Docker approach: ./scripts/soroban-simple.sh --help"
    echo "   3. Manual install: cargo install --locked soroban-cli"
    echo ""
    echo "💡 Recommended: Use option 1 for fastest setup"
    exit 1
fi

# Check network argument
NETWORK="${1:-testnet}"
echo "📡 Deploying to network: $NETWORK"

# Build contracts
echo "🔨 Building project-registry contract..."
cd project-registry
cargo build --target wasm32-unknown-unknown --release
cd ..

echo "🔨 Building funding-escrow contract..."
cd funding-escrow
cargo build --target wasm32-unknown-unknown --release
cd ..

echo "🔨 Building milestone-manager contract..."
cd milestone-manager
cargo build --target wasm32-unknown-unknown --release
cd ..

# Optimize WASM files
echo "⚡ Optimizing WASM files..."
soroban contract optimize \
    --wasm project-registry/target/wasm32-unknown-unknown/release/project_registry.wasm \
    --wasm-out project-registry/target/wasm32-unknown-unknown/release/project_registry_optimized.wasm

soroban contract optimize \
    --wasm funding-escrow/target/wasm32-unknown-unknown/release/funding_escrow.wasm \
    --wasm-out funding-escrow/target/wasm32-unknown-unknown/release/funding_escrow_optimized.wasm

soroban contract optimize \
    --wasm milestone-manager/target/wasm32-unknown-unknown/release/milestone_manager.wasm \
    --wasm-out milestone-manager/target/wasm32-unknown-unknown/release/milestone_manager_optimized.wasm

# Deploy contracts
echo "📦 Deploying project-registry contract..."
PROJECT_REGISTRY_ID=$(soroban contract deploy \
    --wasm project-registry/target/wasm32-unknown-unknown/release/project_registry_optimized.wasm \
    --source-account default \
    --network $NETWORK)

echo "✅ Project Registry deployed: $PROJECT_REGISTRY_ID"

echo "📦 Deploying funding-escrow contract..."
FUNDING_ESCROW_ID=$(soroban contract deploy \
    --wasm funding-escrow/target/wasm32-unknown-unknown/release/funding_escrow_optimized.wasm \
    --source-account default \
    --network $NETWORK)

echo "✅ Funding Escrow deployed: $FUNDING_ESCROW_ID"

echo "📦 Deploying milestone-manager contract..."
MILESTONE_MANAGER_ID=$(soroban contract deploy \
    --wasm milestone-manager/target/wasm32-unknown-unknown/release/milestone_manager_optimized.wasm \
    --source-account default \
    --network $NETWORK)

echo "✅ Milestone Manager deployed: $MILESTONE_MANAGER_ID"

# Save contract addresses to file
echo "💾 Saving contract addresses..."
cat > contract-addresses.json <<EOF
{
  "network": "$NETWORK",
  "contracts": {
    "project_registry": "$PROJECT_REGISTRY_ID",
    "funding_escrow": "$FUNDING_ESCROW_ID",
    "milestone_manager": "$MILESTONE_MANAGER_ID"
  },
  "deployed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF

echo "✅ Contract addresses saved to contract-addresses.json"
echo ""
echo "🎉 Deployment complete!"
echo ""
echo "Contract Addresses:"
echo "  Project Registry: $PROJECT_REGISTRY_ID"
echo "  Funding Escrow:   $FUNDING_ESCROW_ID"
echo "  Milestone Manager: $MILESTONE_MANAGER_ID"
echo ""
echo "Next steps:"
echo "  1. Update your backend .env with these contract addresses"
echo "  2. Initialize the funding escrow contract with:"
echo "     soroban contract invoke \\"
echo "       --id $FUNDING_ESCROW_ID \\"
echo "       --source-account default \\"
echo "       --network $NETWORK \\"
echo "       -- initialize \\"
echo "       --token <USDC_TOKEN_ADDRESS> \\"
echo "       --attestation_pubkey <YOUR_ATTESTATION_PUBKEY>"

