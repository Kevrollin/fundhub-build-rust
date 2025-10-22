#!/usr/bin/env bash
set -euo pipefail

DB_URL="${DATABASE_URL:-}"
if [ -z "$DB_URL" ]; then
  echo "DATABASE_URL is not set" >&2
  exit 1
fi

echo "Applying migrations in migrations/ -> $DB_URL"

# Create migrations tracking table if not exists
psql "$DB_URL" -v ON_ERROR_STOP=1 -c "CREATE TABLE IF NOT EXISTS schema_migrations (filename TEXT PRIMARY KEY, applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW());"

for f in $(ls -1 migrations/*.sql | sort); do
  fname=$(basename "$f")
  # Skip if already applied
  already=$(psql "$DB_URL" -At -c "SELECT 1 FROM schema_migrations WHERE filename='$fname' LIMIT 1;") || already=""
  if [ "$already" = "1" ]; then
    echo "Skipping $fname (already applied)"
    continue
  fi
  echo "Running $fname"
  psql "$DB_URL" -v ON_ERROR_STOP=1 -f "$f"
  psql "$DB_URL" -v ON_ERROR_STOP=1 -c "INSERT INTO schema_migrations (filename) VALUES ('$fname') ON CONFLICT (filename) DO NOTHING;"
done

echo "Migrations applied successfully."

