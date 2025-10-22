#!/bin/bash
set -e

echo "🗄️  Setting up FundHub Database..."

# Start PostgreSQL if not running
echo "Starting PostgreSQL..."
docker-compose up -d postgres

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL..."
sleep 5

# Check if database exists, create if not
docker-compose exec -T postgres psql -U fundhub -lqt | cut -d \| -f 1 | grep -qw fundhub || \
  docker-compose exec -T postgres createdb -U fundhub fundhub

# Export DATABASE_URL
export DATABASE_URL="postgresql://fundhub:fundhub123@localhost:5432/fundhub"

# Run migrations
echo "Running migrations..."
./scripts/run_migrations.sh

echo "✅ Database setup complete!"
echo ""
echo "Now you can run: cargo build"

