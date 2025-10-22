# FundHub Quick Start Guide

Get FundHub backend running in 5 minutes!

## Prerequisites

Ensure you have installed:
- [Rust](https://rustup.rs/) (1.75+)
- [Docker](https://docs.docker.com/get-docker/)
- [Docker Compose](https://docs.docker.com/compose/install/)

## Step-by-Step Setup

### 1. Clone & Navigate

```bash
cd /home/dev-mk/Desktop/Projects/fundhub-build
```

### 2. Create Environment File

```bash
cat > .env << 'EOF'
DATABASE_URL=postgresql://fundhub:fundhub123@localhost:5432/fundhub
REDIS_URL=redis://localhost:6379
MINIO_ENDPOINT=http://localhost:9000
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=minioadmin
JWT_SECRET=dev-secret-change-in-production
JWT_EXP_SECONDS=3600
JWT_REFRESH_EXP_DAYS=30
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
PLATFORM_WALLET_PUBLIC_KEY=GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
RUST_LOG=info,fundhub=debug
EOF
```

### 3. Start Services

```bash
# Start PostgreSQL, Redis, and MinIO
docker-compose up -d postgres redis minio

# Wait for services to be ready (about 10 seconds)
sleep 10
```

### 4. Run Migrations

```bash
export DATABASE_URL="postgresql://fundhub:fundhub123@localhost:5432/fundhub"
./scripts/run_migrations.sh
```

### 5. Start the API

```bash
cargo run
```

🎉 **Done!** The API is now running at http://localhost:8000

## Quick Test

### Test Health Endpoint

```bash
curl http://localhost:8000/health
# Expected: OK
```

### Create a User

```bash
curl -X POST http://localhost:8000/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "SecurePassword123!"
  }'
```

### Login

```bash
curl -X POST http://localhost:8000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "SecurePassword123!"
  }'
```

Save the `access_token` from the response!

## Run the Indexer (Optional)

In a new terminal:

```bash
export DATABASE_URL="postgresql://fundhub:fundhub123@localhost:5432/fundhub"
cargo run --bin indexer
```

## Run Tests

```bash
cargo test --workspace
```

## View Logs

```bash
# API logs
docker-compose logs -f api

# Database logs
docker-compose logs -f postgres

# All logs
docker-compose logs -f
```

## Stop Services

```bash
# Stop API (Ctrl+C in the terminal where it's running)

# Stop Docker services
docker-compose down

# Stop and remove volumes (caution: deletes data!)
docker-compose down -v
```

## Troubleshooting

### Port Already in Use

```bash
# Check what's using port 8000
lsof -i :8000

# Kill the process
kill -9 <PID>
```

### Database Connection Error

```bash
# Restart PostgreSQL
docker-compose restart postgres

# Check if it's running
docker-compose ps postgres
```

### Migration Error

```bash
# Reset database (development only!)
psql postgresql://fundhub:fundhub123@localhost:5432/fundhub \
  -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

# Rerun migrations
./scripts/run_migrations.sh
```

## Next Steps

1. **Deploy Soroban Contracts**
   ```bash
   cd contracts
   ./deploy.sh testnet
   ```

2. **Explore the API**
   - Check out `BACKEND_DOCUMENTATION.md` for detailed API docs
   - Use Postman/Insomnia with the API endpoints
   - Review `IMPLEMENTATION_SUMMARY.md` for architecture

3. **Create Test Data**
   ```bash
   # Create admin user
   # Create students
   # Create projects
   # Test donation flow
   ```

4. **Monitor with Docker**
   ```bash
   # View real-time logs
   docker-compose logs -f api

   # Check service health
   docker-compose ps
   ```

## Development Tips

### Hot Reload

```bash
# Install cargo-watch
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run
```

### Database GUI

```bash
# Use pgAdmin or DBeaver
# Connection: localhost:5432
# Database: fundhub
# User: fundhub
# Password: fundhub123
```

### MinIO Console

Open http://localhost:9001 in your browser
- Username: minioadmin
- Password: minioadmin

## Production Deployment

For production deployment, see `README.md` for:
- Environment configuration
- Security hardening
- Docker production setup
- CI/CD pipeline

## Support

- 📖 Full docs: `README.md`
- 🏗️ Architecture: `IMPLEMENTATION_SUMMARY.md`
- 🔧 Backend API: `BACKEND_DOCUMENTATION.md`

---

**Happy coding! 🚀**

