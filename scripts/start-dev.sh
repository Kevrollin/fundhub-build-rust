#!/bin/bash

set -e

echo "🚀 Starting FundHub Development Environment..."

# Check if .env exists, if not copy from .env.example
if [ ! -f .env ]; then
    echo "📝 Creating .env file from .env.example..."
    cp .env.example .env
    echo "⚠️  Please update .env with your configuration before running again"
    exit 1
fi

# Source environment variables
export $(grep -v '^#' .env | xargs)

# Start docker compose services
echo "🐳 Starting Docker services..."
docker-compose up -d postgres redis minio

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
until docker-compose exec -T postgres pg_isready -U fundhub > /dev/null 2>&1; do
    sleep 1
done

echo "✅ PostgreSQL is ready!"

# Run migrations
echo "🔄 Running database migrations..."
./scripts/run-migrations.sh

# Build and start the API
echo "🔨 Building API..."
cargo build

echo "🚀 Starting API server..."
cargo run

echo "✅ FundHub is running!"
echo "📡 API available at: http://localhost:8000"
echo "📖 API docs available at: http://localhost:8000/api/docs"
echo "📊 Health check: http://localhost:8000/health"

