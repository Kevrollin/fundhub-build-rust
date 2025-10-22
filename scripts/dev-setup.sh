#!/bin/bash

# FundHub Development Setup Script
# This script sets up a lightweight development environment using Docker

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

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker first."
    exit 1
fi

print_status "Setting up FundHub development environment..."

# Create necessary directories
mkdir -p docker
mkdir -p scripts

print_status "Starting database and cache services..."

# Start only the essential services (database, redis, minio)
docker-compose -f docker-compose.dev.yml up -d postgres redis minio

print_status "Waiting for services to be ready..."

# Wait for services to be healthy
echo "Waiting for PostgreSQL..."
while ! docker-compose -f docker-compose.dev.yml exec postgres pg_isready -U dev_mk -d fundhub2 >/dev/null 2>&1; do
    sleep 2
done

echo "Waiting for Redis..."
while ! docker-compose -f docker-compose.dev.yml exec redis redis-cli ping >/dev/null 2>&1; do
    sleep 2
done

echo "Waiting for MinIO..."
while ! curl -f http://localhost:9000/minio/health/live >/dev/null 2>&1; do
    sleep 2
done

print_success "All services are ready!"

# Run database migrations
print_status "Running database migrations..."
if command -v psql >/dev/null 2>&1; then
    # Use local psql if available
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020000000_initial_schema.sql
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020010000_add_student_verification_cols.sql
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020011000_add_campaigns_and_analytics.sql
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020012000_add_campaign_distributions_and_analytics_tables.sql
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020013000_add_missing_tables_and_columns.sql
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020014000_fix_not_null_constraints.sql
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020015000_add_role_system_and_guest_flow.sql
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020020000_add_smart_contracts_integration.sql
    PGPASSWORD="Kevdev%402025" psql -h localhost -U dev_mk -d fundhub2 -f migrations/20251020030000_add_payment_providers.sql
else
    # Use Docker to run migrations
    print_warning "psql not found locally, using Docker to run migrations..."
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020000000_initial_schema.sql
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020010000_add_student_verification_cols.sql
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020011000_add_campaigns_and_analytics.sql
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020012000_add_campaign_distributions_and_analytics_tables.sql
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020013000_add_missing_tables_and_columns.sql
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020014000_fix_not_null_constraints.sql
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020015000_add_role_system_and_guest_flow.sql
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020020000_add_smart_contracts_integration.sql
    docker-compose -f docker-compose.dev.yml exec postgres psql -U dev_mk -d fundhub2 -f /workspace/migrations/20251020030000_add_payment_providers.sql
fi

print_success "Database migrations completed!"

# Build Soroban CLI Docker image (lightweight)
print_status "Building Soroban CLI Docker image (this may take a few minutes)..."
docker build -f docker/soroban.Dockerfile -t fundhub-soroban:latest . || {
    print_warning "Soroban CLI build failed, but you can continue without it for now"
}

print_success "Development environment setup completed!"

echo ""
echo "🎉 FundHub Development Environment Ready!"
echo ""
echo "📊 Services Running:"
echo "   • PostgreSQL: localhost:5432"
echo "   • Redis: localhost:6379"
echo "   • MinIO: localhost:9000 (admin: minioadmin/minioadmin)"
echo ""
echo "🚀 Next Steps:"
echo "   1. Run the API server: cargo run"
echo "   2. Test contracts: ./scripts/contracts-docker.sh testnet"
echo "   3. Configure payment providers in .env"
echo ""
echo "💡 Useful Commands:"
echo "   • View logs: docker-compose -f docker-compose.dev.yml logs -f"
echo "   • Stop services: docker-compose -f docker-compose.dev.yml down"
echo "   • Restart services: docker-compose -f docker-compose.dev.yml restart"
echo ""
echo "🔧 Environment Variables to Set:"
echo "   export DATABASE_URL='postgres://dev_mk:Kevdev%402025@localhost:5432/fundhub2'"
echo "   export REDIS_URL='redis://localhost:6379'"
echo "   export JWT_SECRET='your-jwt-secret-key-here'"
echo "   export STELLAR_HORIZON_URL='https://horizon-testnet.stellar.org'"
