# FundHub Backend

A decentralized crowdfunding platform for student projects, built with Rust, Axum, and Stellar blockchain.

## Features

- **Authentication**: JWT-based auth with refresh tokens and email verification
- **Student Verification**: Document upload and admin approval workflow
- **Wallet Management**: Stellar wallet connection and balance sync
- **Project Management**: Full CRUD with milestones and admin approval
- **Donations**: Memo-based donation tracking with Horizon verification
- **Campaigns**: Automated fund distribution to eligible students
- **Analytics**: Real-time metrics and analytics
- **SSE Notifications**: Real-time server-sent events for updates
- **Background Workers**: Donation verification, wallet sync, analytics collection
- **Soroban Contracts**: On-chain project registry and escrow

## Tech Stack

- **Backend**: Rust, Axum, SQLx, PostgreSQL
- **Blockchain**: Stellar (Testnet), Soroban smart contracts
- **Cache**: Redis
- **Storage**: MinIO (S3-compatible)
- **Auth**: JWT with Argon2 password hashing
- **API Docs**: OpenAPI/Swagger (via utoipa)

## Prerequisites

- Rust 1.75+
- Docker & Docker Compose
- PostgreSQL 15+
- Redis 7+
- Soroban CLI (for contract deployment)

## Quick Start

### 1. Clone the repository

```bash
git clone <repository-url>
cd fundhub-build
```

### 2. Set up environment variables

```bash
cp .env.example .env
# Edit .env with your configuration
```

### 3. Start development environment

```bash
# Make scripts executable
chmod +x scripts/*.sh
chmod +x contracts/deploy.sh

# Start dev environment (PostgreSQL, Redis, MinIO)
./scripts/start-dev.sh
```

This will:
- Start Docker services (PostgreSQL, Redis, MinIO)
- Run database migrations
- Build and start the API server

### 4. Access the application

- **API**: http://localhost:8000
- **Health Check**: http://localhost:8000/health
- **API Docs**: http://localhost:8000/api/docs
- **Swagger UI**: Coming soon with utoipa integration

## Development

### Run migrations manually

```bash
export DATABASE_URL="postgresql://fundhub:fundhub123@localhost:5432/fundhub"
./scripts/run_migrations.sh
```

### Build the project

```bash
cargo build
```

### Run tests

```bash
cargo test --workspace
```

### Run with hot reload

```bash
cargo watch -x run
```

### Lint and format

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings
```

## Soroban Smart Contracts

### Build contracts

```bash
cd contracts

# Build project-registry
cd project-registry
cargo build --target wasm32-unknown-unknown --release

# Build funding-escrow
cd ../funding-escrow
cargo build --target wasm32-unknown-unknown --release
```

### Deploy contracts to testnet

```bash
cd contracts
chmod +x deploy.sh
./deploy.sh testnet
```

This will:
- Build and optimize WASM files
- Deploy both contracts to Stellar testnet
- Save contract addresses to `contract-addresses.json`

### Run contract tests

```bash
cd contracts/project-registry
cargo test

cd ../funding-escrow
cargo test
```

## API Endpoints

### Authentication
- `POST /api/auth/signup` - Register new user
- `POST /api/auth/login` - Login and get tokens
- `POST /api/auth/refresh` - Refresh access token
- `GET /api/auth/verify-email?token=...` - Verify email
- `GET /api/profile/:user_id` - Get user profile

### Students
- `POST /api/students/register` - Submit student verification
- `GET /api/students/status/:user_id` - Check verification status

### Admin
- `GET /api/admin/verifications` - List pending verifications
- `POST /api/admin/verifications/:id/approve` - Approve verification
- `POST /api/admin/verifications/:id/reject` - Reject verification

### Wallets
- `POST /api/wallets/connect` - Connect Stellar wallet
- `GET /api/wallets/balance/:wallet_id` - Get wallet balance
- `GET /api/wallets/transactions/:wallet_id` - Get transaction history

### Projects
- `POST /api/projects` - Create project
- `GET /api/projects` - List projects
- `GET /api/projects/:id` - Get project details
- `PUT /api/projects/:id` - Update project
- `DELETE /api/projects/:id` - Delete project
- `POST /api/projects/:id/publish` - Publish project (admin)
- `POST /api/projects/:id/reject` - Reject project (admin)

### Donations
- `POST /api/donations/initiate` - Initiate donation
- `POST /api/donations/verify` - Verify donation
- `GET /api/donations/project/:project_id` - Get project donations
- `GET /api/donations/student/:student_id` - Get student donations

### Campaigns
- `POST /api/campaigns/create` - Create campaign
- `POST /api/campaigns/execute` - Execute campaign distribution
- `GET /api/campaigns/active` - List active campaigns
- `GET /api/campaigns/:id` - Get campaign details

### Analytics
- `GET /api/analytics/platform/stats` - Platform-wide stats
- `GET /api/analytics/projects/top` - Top projects by donations
- `GET /api/analytics/students/top` - Top students by donations

### Notifications
- `GET /api/notifications/stream` - SSE stream for real-time updates

## Docker Deployment

### Build Docker image

```bash
docker build -t fundhub-api:latest .
```

### Run with Docker Compose

```bash
docker-compose up --build
```

This will start all services:
- API (port 8000)
- PostgreSQL (port 5432)
- Redis (port 6379)
- MinIO (ports 9000, 9001)

## Background Workers

The following workers run automatically:

1. **Donation Verifier** (every 2 minutes)
   - Polls Horizon for pending donation transactions
   - Verifies amounts and memos
   - Updates donation status

2. **Wallet Sync** (every 5 minutes)
   - Syncs wallet balances from Stellar network
   - Updates balance in database

3. **Analytics Collector** (every 10 minutes)
   - Aggregates donation metrics
   - Updates analytics summaries

4. **Campaign Distributor**
   - Executes active campaigns
   - Distributes funds to eligible recipients

## Testing

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test '*'
```

### Contract Tests

```bash
cd contracts/project-registry && cargo test
cd ../funding-escrow && cargo test
```

## CI/CD

GitHub Actions workflow runs on push and PR:
- Runs all tests
- Checks code formatting
- Runs clippy linter
- Builds Soroban contracts
- Builds Docker image

## Environment Variables

See `.env.example` for all available configuration options.

### Required Variables

- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `JWT_SECRET` - Secret for JWT signing
- `STELLAR_HORIZON_URL` - Stellar Horizon API URL
- `PLATFORM_WALLET_PUBLIC_KEY` - Platform wallet public key

### Optional Variables

- `MINIO_*` - MinIO/S3 configuration
- `SMTP_*` - Email configuration
- `WORKER_*` - Worker interval configuration

## Production Deployment

### Security Checklist

- [ ] Change all default passwords and secrets
- [ ] Use KMS for wallet private keys (don't store in env)
- [ ] Enable rate limiting
- [ ] Configure CORS properly
- [ ] Set up SSL/TLS certificates
- [ ] Configure proper logging and monitoring
- [ ] Enable database backups
- [ ] Set up proper key rotation

### Deployment Steps

1. Set up production database and Redis
2. Configure environment variables
3. Run migrations
4. Deploy contracts to Stellar mainnet
5. Build and deploy Docker image
6. Configure reverse proxy (nginx/caddy)
7. Set up monitoring and alerts

## Troubleshooting

### Database connection issues

```bash
# Check if PostgreSQL is running
docker-compose ps postgres

# Check logs
docker-compose logs postgres
```

### Migration issues

```bash
# Reset database (development only!)
psql $DATABASE_URL -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
./scripts/run_migrations.sh
```

### Worker not running

Check logs for worker errors:
```bash
docker-compose logs api | grep worker
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Support

For issues and questions:
- Create an issue on GitHub
- Contact the development team

## Roadmap

- [ ] Add email notification service
- [ ] Implement milestone-based fund release
- [ ] Add support for multiple currencies
- [ ] Build admin dashboard
- [ ] Add GraphQL API
- [ ] Implement reputation system
- [ ] Add project discovery/search
- [ ] Build mobile app
