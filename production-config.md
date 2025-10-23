# Production Configuration

## Environment Variables for Production Deployment

### Backend (Render) Environment Variables

```bash
# Database
DATABASE_URL=postgresql://user:pass@host:port/db
REDIS_URL=redis://user:pass@host:port

# Security
JWT_SECRET=your-super-secret-jwt-key-here-make-it-long-and-random

# Stellar Network
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
PLATFORM_WALLET_PUBLIC_KEY=your-platform-wallet-public-key

# Server
PORT=8000
RUST_LOG=info

# CORS
CORS_ORIGIN=https://your-frontend-domain.vercel.app

# Optional: File Storage
MINIO_ENDPOINT=your-minio-endpoint
MINIO_ACCESS_KEY=your-access-key
MINIO_SECRET_KEY=your-secret-key
MINIO_BUCKET=your-bucket-name

# Optional: Email
SMTP_HOST=your-smtp-host
SMTP_PORT=587
SMTP_USERNAME=your-smtp-username
SMTP_PASSWORD=your-smtp-password
SMTP_FROM=noreply@yourdomain.com
```

### Frontend (Vercel) Environment Variables

```bash
# API Configuration
VITE_API_BASE_URL=https://your-backend-name.onrender.com/api

# Stellar Configuration
VITE_STELLAR_NETWORK=testnet
VITE_HORIZON_URL=https://horizon-testnet.stellar.org
VITE_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org

# Platform Configuration
VITE_PLATFORM_NAME=FundHub
VITE_PLATFORM_DESCRIPTION=Decentralized crowdfunding for student projects

# Feature Flags
VITE_ENABLE_ANALYTICS=true
VITE_ENABLE_NOTIFICATIONS=true
VITE_ENABLE_WALLET_CONNECTION=true
```

## Security Checklist

- [ ] Use strong, unique JWT secrets
- [ ] Enable HTTPS (automatic on Render/Vercel)
- [ ] Configure proper CORS settings
- [ ] Use environment variables for all secrets
- [ ] Enable database backups
- [ ] Set up monitoring and alerts
- [ ] Regular security updates

## Performance Optimization

- [ ] Enable Redis caching
- [ ] Optimize database queries
- [ ] Use CDN for static assets (Vercel handles this)
- [ ] Enable compression
- [ ] Monitor resource usage

## Monitoring Setup

- [ ] Set up health checks
- [ ] Monitor database performance
- [ ] Track API response times
- [ ] Set up error alerts
- [ ] Monitor resource usage
