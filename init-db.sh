#!/bin/bash

# Database initialization script for Render
# This script runs migrations when the service starts

set -e

echo "🚀 Initializing database..."

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ Error: DATABASE_URL environment variable is not set"
    exit 1
fi

# Wait for database to be ready
echo "⏳ Waiting for database to be ready..."
until psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1; do
    echo "  Database not ready, waiting..."
    sleep 2
done

echo "✅ Database is ready"

# Run migrations
echo "📝 Running database migrations..."
for migration in migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo "  Running migration: $(basename "$migration")"
        psql "$DATABASE_URL" -f "$migration" || echo "  Migration may have already been applied"
    fi
done

echo "✅ Database initialization complete!"
