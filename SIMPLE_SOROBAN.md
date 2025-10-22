# Simple Soroban CLI via Docker

## 🐳 **Lightweight Soroban CLI Setup**

This approach gives you Soroban CLI via Docker without heavy local installations or complex Docker Compose setups.

## 🚀 **Quick Start**

### 1. **Build Soroban CLI Docker Image**
```bash
# This builds the Soroban CLI in Docker (one-time setup)
docker build -f docker/soroban.Dockerfile -t fundhub-soroban:latest .
```

### 2. **Use Soroban CLI**
```bash
# Run any Soroban command via Docker
./scripts/soroban-simple.sh --help
./scripts/soroban-simple.sh config network add testnet --rpc-url https://soroban-rpc.testnet.stellar.org --network-passphrase "Test SDF Network ; September 2015"
./scripts/soroban-simple.sh config identity generate fundhub-admin
```

### 3. **Build and Deploy Contracts**
```bash
# Build contracts and get deployment instructions
./scripts/deploy-contracts-simple.sh testnet
```

## 📋 **Available Scripts**

| Script | Purpose |
|--------|---------|
| `./scripts/soroban-simple.sh` | Run any Soroban CLI command via Docker |
| `./scripts/deploy-contracts-simple.sh` | Build contracts and show deployment steps |

## 🔧 **Common Commands**

### **Soroban CLI Commands**
```bash
# Get help
./scripts/soroban-simple.sh --help

# Configure network
./scripts/soroban-simple.sh config network add testnet --rpc-url https://soroban-rpc.testnet.stellar.org --network-passphrase "Test SDF Network ; September 2015"

# Create identity
./scripts/soroban-simple.sh config identity generate fundhub-admin

# Fund identity (get testnet XLM from friendbot)
./scripts/soroban-simple.sh config identity fund fundhub-admin --network testnet

# Deploy contract
./scripts/soroban-simple.sh contract deploy --wasm contract.wasm --source-account fundhub-admin --network testnet

# Invoke contract
./scripts/soroban-simple.sh contract invoke --id CONTRACT_ID --source-account fundhub-admin --network testnet -- initialize --token TOKEN_ID
```

### **Contract Development**
```bash
# Build contracts
./scripts/deploy-contracts-simple.sh testnet

# Test contracts locally
./scripts/soroban-simple.sh contract invoke --help
```

## 🎯 **Complete Deployment Workflow**

### **1. Setup Network and Identity**
```bash
# Configure testnet
./scripts/soroban-simple.sh config network add testnet --rpc-url https://soroban-rpc.testnet.stellar.org --network-passphrase "Test SDF Network ; September 2015"

# Create admin identity
./scripts/soroban-simple.sh config identity generate fundhub-admin

# Fund the identity (get testnet XLM)
./scripts/soroban-simple.sh config identity fund fundhub-admin --network testnet
```

### **2. Build Contracts**
```bash
# Build all contracts
./scripts/deploy-contracts-simple.sh testnet
```

### **3. Deploy Contracts**
```bash
# Deploy Project Registry
PROJECT_REGISTRY_ID=$(./scripts/soroban-simple.sh contract deploy --wasm project-registry/target/wasm32-unknown-unknown/release/project_registry_optimized.wasm --source-account fundhub-admin --network testnet)
echo "Project Registry: $PROJECT_REGISTRY_ID"

# Deploy Funding Escrow
FUNDING_ESCROW_ID=$(./scripts/soroban-simple.sh contract deploy --wasm funding-escrow/target/wasm32-unknown-unknown/release/funding_escrow_optimized.wasm --source-account fundhub-admin --network testnet)
echo "Funding Escrow: $FUNDING_ESCROW_ID"

# Deploy Milestone Manager
MILESTONE_MANAGER_ID=$(./scripts/soroban-simple.sh contract deploy --wasm milestone-manager/target/wasm32-unknown-unknown/release/milestone_manager_optimized.wasm --source-account fundhub-admin --network testnet)
echo "Milestone Manager: $MILESTONE_MANAGER_ID"
```

### **4. Initialize Contracts**
```bash
# Initialize Funding Escrow (you'll need a USDC token address)
USDC_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQAHHX4QLW6"  # Testnet USDC
ATTESTATION_KEY="0000000000000000000000000000000000000000000000000000000000000001"

./scripts/soroban-simple.sh contract invoke --id $FUNDING_ESCROW_ID --source-account fundhub-admin --network testnet -- initialize --token $USDC_TOKEN --attestation_pubkey $ATTESTATION_KEY

# Initialize Milestone Manager
./scripts/soroban-simple.sh contract invoke --id $MILESTONE_MANAGER_ID --source-account fundhub-admin --network testnet -- initialize --admin $(./scripts/soroban-simple.sh config identity address fundhub-admin) --attestation_key $ATTESTATION_KEY
```

### **5. Update Backend Configuration**
```bash
# Update your .env file with contract addresses
echo "PROJECT_REGISTRY_CONTRACT=$PROJECT_REGISTRY_ID" >> .env
echo "FUNDING_ESCROW_CONTRACT=$FUNDING_ESCROW_ID" >> .env
echo "MILESTONE_MANAGER_CONTRACT=$MILESTONE_MANAGER_ID" >> .env
```

## 🛠️ **Troubleshooting**

### **Common Issues**

1. **Docker not running**
   ```bash
   sudo systemctl start docker
   ```

2. **Soroban image not found**
   ```bash
   docker build -f docker/soroban.Dockerfile -t fundhub-soroban:latest .
   ```

3. **Contract build fails**
   ```bash
   # Check if Rust target is installed
   rustup target add wasm32-unknown-unknown
   ```

4. **Network connection issues**
   ```bash
   # Test network connectivity
   curl https://soroban-rpc.testnet.stellar.org
   ```

### **Useful Docker Commands**

```bash
# View Docker images
docker images | grep fundhub

# Remove old image
docker rmi fundhub-soroban:latest

# Rebuild image
docker build -f docker/soroban.Dockerfile -t fundhub-soroban:latest . --no-cache

# Run interactive container
docker run --rm -it -v $(pwd):/workspace fundhub-soroban:latest bash
```

## 🎉 **Benefits**

✅ **No heavy local installations** - Soroban CLI runs in Docker  
✅ **Lightweight** - Only builds what you need  
✅ **Isolated** - Doesn't affect your system  
✅ **Consistent** - Same environment every time  
✅ **Easy cleanup** - Just remove Docker image  

This approach gives you **full Soroban CLI functionality** without the system stress of local installation!
