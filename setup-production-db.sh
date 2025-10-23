#!/bin/bash

# Production Database Setup Script
# This script sets up the database for production deployment

set -e

echo "🚀 Setting up production database..."

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ Error: DATABASE_URL environment variable is not set"
    echo "Please set DATABASE_URL to your PostgreSQL connection string"
    echo "Example: export DATABASE_URL='postgresql://user:pass@host:port/db'"
    exit 1
fi

echo "📊 Database URL: ${DATABASE_URL}"

# Test database connection
echo "🔍 Testing database connection..."
if ! psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1; then
    echo "❌ Error: Cannot connect to database"
    echo "Please check your DATABASE_URL and ensure the database is accessible"
    exit 1
fi

echo "✅ Database connection successful"

# Run migrations
echo "📝 Running database migrations..."
for migration in migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo "  Running migration: $(basename "$migration")"
        psql "$DATABASE_URL" -f "$migration"
    fi
done

echo "✅ Database setup complete!"

# Show database info
echo "📊 Database information:"
psql "$DATABASE_URL" -c "
SELECT 
    schemaname,
    tablename,
    tableowner
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY tablename;
"

echo "🎉 Production database is ready!"
