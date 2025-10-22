#!/bin/bash
set -e

echo "🗄️  Setting up FundHub Database (using existing PostgreSQL)..."

# Check if PostgreSQL is accessible
if ! psql -h localhost -p 5432 -U postgres -lqt &> /dev/null; then
    echo "⚠️  Cannot connect to PostgreSQL on localhost:5432"
    echo "Please ensure PostgreSQL is running and accessible."
    echo ""
    echo "Options:"
    echo "1. Start local PostgreSQL: sudo systemctl start postgresql"
    echo "2. Use Docker on different port: Edit docker-compose.yml to use 5433:5432"
    exit 1
fi

echo "✅ PostgreSQL is accessible"

# Create database and user if they don't exist
echo "Creating database and user..."
psql -h localhost -p 5432 -U postgres << 'EOSQL' || true
CREATE USER fundhub WITH PASSWORD 'fundhub123';
CREATE DATABASE fundhub OWNER fundhub;
GRANT ALL PRIVILEGES ON DATABASE fundhub TO fundhub;
EOSQL

echo "✅ Database 'fundhub' created/verified"

# Export DATABASE_URL
export DATABASE_URL="postgresql://fundhub:fundhub123@localhost:5432/fundhub"

# Run migrations
echo "Running migrations..."
./scripts/run_migrations.sh

echo ""
echo "✅ Database setup complete!"
echo ""
echo "DATABASE_URL=postgresql://fundhub:fundhub123@localhost:5432/fundhub"
echo ""
echo "Now you can run: cargo build"

