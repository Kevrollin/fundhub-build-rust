# Soroban CLI Installation Options

## 🚀 **3 Ways to Get Soroban CLI**

Since the Docker build had network issues, here are your options:

### **Option 1: Lightweight Direct Install (RECOMMENDED)**
```bash
# Fastest and most reliable
./scripts/soroban-download.sh
```

**Pros:**
- ✅ Fastest installation
- ✅ No Docker complexity
- ✅ Works offline after install
- ✅ Direct binary access

**Cons:**
- ⚠️ Installs Rust if not present
- ⚠️ Uses some system resources during install

### **Option 2: Docker Approach (If you prefer isolation)**
```bash
# Use Docker for Soroban CLI
./scripts/soroban-simple.sh --help
```

**Pros:**
- ✅ Completely isolated
- ✅ No system changes
- ✅ Easy cleanup

**Cons:**
- ⚠️ Requires Docker
- ⚠️ Slower for repeated use
- ⚠️ Network issues during build

### **Option 3: Manual Installation**
```bash
# Traditional approach
cargo install --locked soroban-cli
```

**Pros:**
- ✅ Standard approach
- ✅ Full control

**Cons:**
- ⚠️ Can be slow and resource-intensive
- ⚠️ May stress your system

## 🎯 **Recommended Workflow**

### **Step 1: Install Soroban CLI**
```bash
# Choose your preferred method
./scripts/soroban-download.sh  # RECOMMENDED
```

### **Step 2: Configure Network**
```bash
# Configure testnet
soroban config network add testnet --rpc-url https://soroban-rpc.testnet.stellar.org --network-passphrase "Test SDF Network ; September 2015"

# Create admin identity
soroban config identity generate fundhub-admin

# Fund the identity (get testnet XLM)
soroban config identity fund fundhub-admin --network testnet
```

### **Step 3: Deploy Contracts**
```bash
# Deploy all contracts
cd contracts && ./deploy.sh testnet
```

## 🔧 **Troubleshooting**

### **If Option 1 fails:**
```bash
# Check if Rust is installed
rustc --version

# Install Rust manually if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env
```

### **If Option 2 fails:**
```bash
# Check Docker
docker --version

# Try with different base image
docker build -f docker/soroban-lite.Dockerfile -t fundhub-soroban:latest .
```

### **If Option 3 is too slow:**
```bash
# Install with minimal features
cargo install --locked soroban-cli --no-default-features

# Or use the lightweight script instead
./scripts/soroban-download.sh
```

## 💡 **Quick Decision Guide**

**Choose Option 1 if:**
- You want the fastest setup
- You don't mind installing Rust
- You want direct access to Soroban CLI

**Choose Option 2 if:**
- You prefer Docker isolation
- You don't want to install anything locally
- You're comfortable with Docker

**Choose Option 3 if:**
- You want full control
- You're comfortable with long build times
- You prefer traditional installation

## 🎉 **After Installation**

Once you have Soroban CLI working:

```bash
# Test it works
soroban --version

# Deploy contracts
cd contracts && ./deploy.sh testnet

# Continue with FundHub development
cargo run
```

The **lightweight direct install (Option 1)** is usually the best choice for development!
