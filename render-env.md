# Render Backend Environment Variables

Set these environment variables in your Render dashboard:

## Required Variables

### Database
- `DATABASE_URL`: PostgreSQL connection string
  - Format: `postgresql://username:password@hostname:port/database_name`
  - Example: `postgresql://fundhub:password123@dpg-abc123-a.oregon-postgres.render.com:5432/fundhub_db`

### Redis
- `REDIS_URL`: Redis connection string
  - Format: `redis://username:password@hostname:port`
  - Example: `redis://red-abc123:password123@oregon-redis.render.com:6379`

### Authentication
- `JWT_SECRET`: Secret key for JWT token signing
  - Generate a strong random string (32+ characters)
  - Example: `your-super-secret-jwt-key-here-make-it-long-and-random`

### Stellar Configuration
- `STELLAR_NETWORK`: `testnet` or `mainnet`
- `STELLAR_HORIZON_URL`: Stellar Horizon API URL
  - Testnet: `https://horizon-testnet.stellar.org`
  - Mainnet: `https://horizon.stellar.org`
- `SOROBAN_RPC_URL`: Soroban RPC URL
  - Testnet: `https://soroban-testnet.stellar.org`
  - Mainnet: `https://soroban-mainnet.stellar.org`
- `PLATFORM_WALLET_PUBLIC_KEY`: Your platform wallet public key

### Server Configuration
- `PORT`: Server port (Render sets this automatically)
- `RUST_LOG`: Logging level (`info`, `debug`, `warn`, `error`)

## Optional Variables

### File Storage (MinIO/S3)
- `MINIO_ENDPOINT`: MinIO endpoint URL
- `MINIO_ACCESS_KEY`: MinIO access key
- `MINIO_SECRET_KEY`: MinIO secret key
- `MINIO_BUCKET`: MinIO bucket name

### Email Configuration
- `SMTP_HOST`: SMTP server hostname
- `SMTP_PORT`: SMTP server port (usually 587)
- `SMTP_USERNAME`: SMTP username
- `SMTP_PASSWORD`: SMTP password
- `SMTP_FROM`: From email address

## How to Set Environment Variables in Render

1. Go to your service dashboard in Render
2. Click on "Environment" tab
3. Add each variable with its value
4. Click "Save Changes"
5. Redeploy your service
