# FundHub Docker Development Setup

## 🐳 **Lightweight Docker Development**

This setup uses Docker to avoid heavy local installations while keeping your system responsive.

## 🚀 **Quick Start**

### 1. **Setup Development Environment**
```bash
# Run the lightweight development setup
./scripts/dev-setup.sh
```

This will:
- Start PostgreSQL, Redis, and MinIO in Docker
- Run all database migrations
- Build Soroban CLI in Docker (lightweight)
- Set up the development environment

### 2. **Start the API Server**
```bash
# Set environment variables
export DATABASE_URL='postgres://dev_mk:Kevdev%402025@localhost:5432/fundhub2'
export REDIS_URL='redis://localhost:6379'
export JWT_SECRET='your-jwt-secret-key-here'
export STELLAR_HORIZON_URL='https://horizon-testnet.stellar.org'

# Run the API server locally (lightweight)
cargo run
```

### 3. **Test Smart Contracts (Docker-based)**
```bash
# Build and test contracts using Docker
./scripts/contracts-docker.sh testnet
```

## 📊 **Services Overview**

| Service | Port | Purpose | Credentials |
|---------|------|---------|-------------|
| PostgreSQL | 5432 | Database | dev_mk / Kevdev%402025 |
| Redis | 6379 | Cache/Queue | - |
| MinIO | 9000 | Object Storage | minioadmin / minioadmin |
| MinIO Console | 9001 | Web UI | minioadmin / minioadmin |

## 🔧 **Docker Commands**

### **Development Services**
```bash
# Start all services
docker-compose -f docker-compose.dev.yml up -d

# Start specific services
docker-compose -f docker-compose.dev.yml up -d postgres redis minio

# View logs
docker-compose -f docker-compose.dev.yml logs -f

# Stop services
docker-compose -f docker-compose.dev.yml down
```

### **Soroban CLI (Docker-based)**
```bash
# Build Soroban CLI image
docker build -f docker/soroban.Dockerfile -t fundhub-soroban:latest .

# Run Soroban CLI commands
docker run --rm -v $(pwd):/workspace fundhub-soroban:latest --help

# Deploy contracts
./scripts/contracts-docker.sh testnet
```

## 🎯 **Development Workflow**

### **1. Database Development**
```bash
# Connect to database
docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2

# Run migrations
./scripts/run_migrations.sh

# Reset database
docker-compose -f docker-compose.dev.yml down -v
docker-compose -f docker-compose.dev.yml up -d postgres redis minio
./scripts/dev-setup.sh
```

### **2. API Development**
```bash
# Run with hot reload (if you have cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Run tests
cargo test

# Check linting
cargo clippy
```

### **3. Smart Contract Development**
```bash
# Build contracts
./scripts/contracts-docker.sh testnet

# Test contracts locally
docker run --rm -v $(pwd):/workspace fundhub-soroban:latest contract invoke --help
```

## 🔒 **Environment Configuration**

Create a `.env` file in the project root:

```bash
# Database
DATABASE_URL=postgres://dev_mk:Kevdev%402025@localhost:5432/fundhub2
REDIS_URL=redis://localhost:6379

# Authentication
JWT_SECRET=your-super-secret-jwt-key-here

# Stellar
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
PLATFORM_WALLET_PUBLIC_KEY=your-platform-wallet-key

# MinIO
MINIO_ENDPOINT=http://localhost:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin

# Payment Providers (Optional)
MPESA_CONSUMER_KEY=your_mpesa_key
MPESA_CONSUMER_SECRET=your_mpesa_secret
MPESA_BUSINESS_SHORT_CODE=your_short_code
MPESA_PASSKEY=your_passkey
MPESA_CALLBACK_URL=https://your-domain.com/api/payments/mpesa/webhook
MPESA_ENVIRONMENT=sandbox

STRIPE_SECRET_KEY=sk_test_...
STRIPE_PUBLISHABLE_KEY=pk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_SUCCESS_URL=https://your-domain.com/success
STRIPE_CANCEL_URL=https://your-domain.com/cancel
```

## 🚀 **Production Deployment**

### **Docker Compose Production**
```bash
# Build production images
docker-compose -f docker-compose.prod.yml build

# Deploy to production
docker-compose -f docker-compose.prod.yml up -d
```

### **Kubernetes Deployment**
```bash
# Apply Helm charts
helm install fundhub ./helm/fundhub

# Deploy contracts
./scripts/contracts-docker.sh mainnet
```

## 🛠️ **Troubleshooting**

### **Common Issues**

1. **Docker not running**
   ```bash
   sudo systemctl start docker
   ```

2. **Port conflicts**
   ```bash
   # Check what's using the ports
   sudo netstat -tulpn | grep :5432
   sudo netstat -tulpn | grep :6379
   sudo netstat -tulpn | grep :9000
   ```

3. **Database connection issues**
   ```bash
   # Check if PostgreSQL is running
   docker-compose -f docker-compose.dev.yml exec postgres pg_isready -U dev_mk -d fundhub2
   ```

4. **Soroban CLI issues**
   ```bash
   # Rebuild Soroban image
   docker build -f docker/soroban.Dockerfile -t fundhub-soroban:latest . --no-cache
   ```

### **Performance Optimization**

1. **Docker Resource Limits**
   ```yaml
   # In docker-compose.dev.yml
   services:
     postgres:
       deploy:
         resources:
           limits:
             memory: 512M
             cpus: '0.5'
   ```

2. **Cargo Cache**
   ```bash
   # Use Docker volume for cargo cache
   docker volume create cargo-cache
   ```

## 📚 **Useful Commands**

```bash
# View all containers
docker ps

# View container logs
docker logs <container_name>

# Execute commands in container
docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2

# Clean up Docker resources
docker system prune -a

# View Docker resource usage
docker stats
```

## 🎉 **Benefits of Docker Development**

✅ **No heavy local installations** - Soroban CLI builds in Docker  
✅ **Consistent environment** - Same setup across all machines  
✅ **Easy cleanup** - Just `docker-compose down -v` to reset  
✅ **Resource isolation** - Services don't interfere with your system  
✅ **Production parity** - Same services as production environment  

This setup gives you a **lightweight, efficient development environment** that won't stress your system while providing all the functionality you need for FundHub development!
