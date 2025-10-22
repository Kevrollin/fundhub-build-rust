# Build Instructions

## Quick Fix for SQLx Compile-Time Verification Error

You're seeing errors because SQLx tries to verify queries against the database at compile time. Here are your options:

### **Option 1: Setup Database First** (Recommended)

```bash
# 1. Create .env file
cat > .env << 'EOF'
DATABASE_URL=postgresql://fundhub:fundhub123@localhost:5432/fundhub
REDIS_URL=redis://localhost:6379
JWT_SECRET=dev-secret-change-in-production
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
PLATFORM_WALLET_PUBLIC_KEY=GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
RUST_LOG=info,fundhub=debug
EOF

# 2. Start PostgreSQL
docker-compose up -d postgres

# 3. Wait for it to be ready
sleep 10

# 4. Run migrations
export DATABASE_URL="postgresql://fundhub:fundhub123@localhost:5432/fundhub"
./scripts/run_migrations.sh

# 5. Now build!
cargo build
```

### **Option 2: Use SQLx Offline Mode**

If you don't want to set up the database yet:

```bash
# Skip compile-time verification (not recommended for production)
export SQLX_OFFLINE=true
cargo build
```

### **Option 3: Generate SQLx Offline Data**

After setting up the database (Option 1):

```bash
# This creates .sqlx/ directory with query metadata
cargo sqlx prepare

# Now you can build without database connection
export SQLX_OFFLINE=true
cargo build
```

## Recommended Flow

1. **First time setup:**
   ```bash
   chmod +x setup-db.sh
   ./setup-db.sh
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run the application:**
   ```bash
   cargo run
   ```

4. **Run tests:**
   ```bash
   cargo test
   ```

## Troubleshooting

### "relation does not exist" errors

This means migrations haven't run. Solution:
```bash
export DATABASE_URL="postgresql://fundhub:fundhub123@localhost:5432/fundhub"
./scripts/run_migrations.sh
```

### PostgreSQL not running

```bash
docker-compose up -d postgres
# Wait 10 seconds
docker-compose ps postgres
```

### Can't connect to PostgreSQL

Check your DATABASE_URL matches the docker-compose.yml settings:
- Host: localhost
- Port: 5432
- User: fundhub
- Password: fundhub123
- Database: fundhub

## Next Steps After Successful Build

1. Start all services: `docker-compose up -d`
2. Run migrations: `./scripts/run_migrations.sh`
3. Start API: `cargo run`
4. Test: `curl http://localhost:8000/health`

