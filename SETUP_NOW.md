# Quick Setup - Do This Now

Since PostgreSQL is already running on port 5432, follow these exact steps:

## Step 1: Create Database and User

```bash
# Connect to PostgreSQL as superuser
sudo -u postgres psql

# Then run these commands in the PostgreSQL prompt:
CREATE USER fundhub WITH PASSWORD 'fundhub123';
CREATE DATABASE fundhub OWNER fundhub;
GRANT ALL PRIVILEGES ON DATABASE fundhub TO fundhub;
\q
```

## Step 2: Set Environment Variable

```bash
export DATABASE_URL="postgresql://fundhub:fundhub123@localhost:5432/fundhub"
```

## Step 3: Run Migrations

```bash
./scripts/run_migrations.sh
```

## Step 4: Build the Project

```bash
# Unset offline mode if it was set
unset SQLX_OFFLINE

# Remove the offline config if it exists
rm -f .cargo/config.toml

# Now build
cargo build
```

## If You Still See SQLX_OFFLINE Error

Run this to check:
```bash
env | grep SQLX
```

If you see `SQLX_OFFLINE=true`, unset it:
```bash
unset SQLX_OFFLINE
cargo clean
cargo build
```

## Quick Alternative - Skip SQLx Verification Entirely

If you just want to build without database setup:

```bash
# This disables compile-time verification
cargo build --no-default-features
```

---

## What's Next After Successful Build?

1. **Run the API:**
   ```bash
   cargo run
   ```

2. **Test it:**
   ```bash
   curl http://localhost:8000/health
   ```

3. **Create a user:**
   ```bash
   curl -X POST http://localhost:8000/api/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"username":"test","email":"test@example.com","password":"Test123!"}'
   ```

