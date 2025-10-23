# FundHub Deployment Guide

This guide will help you deploy your FundHub platform to Render (backend) and Vercel (frontend).

## 🚀 Quick Deployment Overview

1. **Backend**: Deploy Rust API to Render
2. **Frontend**: Deploy React app to Vercel
3. **Database**: Set up PostgreSQL and Redis on Render
4. **Configuration**: Set up environment variables

## 📋 Prerequisites

- GitHub repository with your code
- Render account (free tier available)
- Vercel account (free tier available)
- Stellar wallet for platform operations

## 🔧 Backend Deployment (Render)

### Step 1: Prepare Your Repository

1. Ensure all files are committed to your GitHub repository
2. Make sure the following files exist:
   - `render.yaml` (Render configuration)
   - `init-db.sh` (Database initialization)
   - `migrations/` (Database migrations)

### Step 2: Create Render Service

1. Go to [Render Dashboard](https://dashboard.render.com)
2. Click "New +" → "Web Service"
3. Connect your GitHub repository
4. Configure the service:
   - **Name**: `fundhub-backend`
   - **Environment**: `Rust`
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./init-db.sh && ./target/release/fundhub`
   - **Plan**: Starter (Free)

### Step 3: Set Up Database

1. In Render dashboard, go to "New +" → "PostgreSQL"
2. Configure:
   - **Name**: `fundhub-db`
   - **Database**: `fundhub`
   - **User**: `fundhub`
   - **Plan**: Starter (Free)
3. Copy the connection string

### Step 4: Set Up Redis

1. In Render dashboard, go to "New +" → "Redis"
2. Configure:
   - **Name**: `fundhub-redis`
   - **Plan**: Starter (Free)
3. Copy the connection string

### Step 5: Configure Environment Variables

In your Render service settings, add these environment variables:

#### Required Variables
```
DATABASE_URL=postgresql://user:pass@host:port/db
REDIS_URL=redis://user:pass@host:port
JWT_SECRET=your-super-secret-jwt-key-here
STELLAR_NETWORK=testnet
STELLAR_HORIZON_URL=https://horizon-testnet.stellar.org
SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
PLATFORM_WALLET_PUBLIC_KEY=your-platform-wallet-public-key
RUST_LOG=info
```

#### Optional Variables
```
MINIO_ENDPOINT=your-minio-endpoint
MINIO_ACCESS_KEY=your-access-key
MINIO_SECRET_KEY=your-secret-key
SMTP_HOST=your-smtp-host
SMTP_USERNAME=your-smtp-username
SMTP_PASSWORD=your-smtp-password
```

### Step 6: Deploy

1. Click "Create Web Service"
2. Render will automatically build and deploy your service
3. Wait for deployment to complete
4. Note your service URL (e.g., `https://fundhub-backend.onrender.com`)

## 🎨 Frontend Deployment (Vercel)

### Step 1: Prepare Frontend

1. Ensure your frontend code is in the `fundbloom-nexus/` directory
2. Make sure `vercel.json` exists in the frontend directory

### Step 2: Deploy to Vercel

1. Go to [Vercel Dashboard](https://vercel.com/dashboard)
2. Click "New Project"
3. Import your GitHub repository
4. Configure the project:
   - **Framework Preset**: Vite
   - **Root Directory**: `fundbloom-nexus`
   - **Build Command**: `npm run build`
   - **Output Directory**: `dist`

### Step 3: Set Environment Variables

In Vercel project settings, add these environment variables:

```
VITE_API_BASE_URL=https://your-backend-name.onrender.com/api
VITE_STELLAR_NETWORK=testnet
VITE_HORIZON_URL=https://horizon-testnet.stellar.org
VITE_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
VITE_PLATFORM_NAME=FundHub
VITE_PLATFORM_DESCRIPTION=Decentralized crowdfunding for student projects
```

### Step 4: Deploy

1. Click "Deploy"
2. Vercel will build and deploy your frontend
3. Note your frontend URL (e.g., `https://your-app-name.vercel.app`)

## 🔗 Connect Frontend and Backend

### Step 1: Update CORS Settings

1. Go to your Render service
2. Update the `CORS_ORIGIN` environment variable with your Vercel URL:
```
CORS_ORIGIN=https://your-app-name.vercel.app
```

### Step 2: Update Frontend API URL

1. In Vercel, update the `VITE_API_BASE_URL` environment variable:
```
VITE_API_BASE_URL=https://your-backend-name.onrender.com/api
```

### Step 3: Redeploy Both Services

1. Redeploy your Render service
2. Redeploy your Vercel service

## 🧪 Testing Your Deployment

### Backend Health Check
```bash
curl https://your-backend-name.onrender.com/health
```

### Frontend Access
Visit your Vercel URL to access the frontend application.

### API Documentation
Visit `https://your-backend-name.onrender.com/api/docs` for API documentation.

## 🔧 Troubleshooting

### Common Issues

1. **Database Connection Errors**
   - Check `DATABASE_URL` format
   - Ensure database is accessible
   - Verify migrations ran successfully

2. **CORS Errors**
   - Update CORS settings in backend
   - Check frontend URL in CORS configuration

3. **Build Failures**
   - Check Rust version compatibility
   - Verify all dependencies are available
   - Check build logs in Render dashboard

4. **Environment Variable Issues**
   - Ensure all required variables are set
   - Check variable names and values
   - Restart services after changes

### Debugging Steps

1. **Check Logs**
   - Render: Service logs in dashboard
   - Vercel: Function logs in dashboard

2. **Test Endpoints**
   - Use curl or Postman to test API endpoints
   - Check browser developer tools for frontend errors

3. **Database Issues**
   - Connect to database directly
   - Check migration status
   - Verify table creation

## 📊 Monitoring and Maintenance

### Render Monitoring
- Monitor service health in Render dashboard
- Set up alerts for service downtime
- Monitor database performance

### Vercel Monitoring
- Monitor deployment status
- Check function performance
- Monitor build times

### Database Maintenance
- Regular backups (Render handles this automatically)
- Monitor connection limits
- Check query performance

## 🔄 Updates and Redeployment

### Backend Updates
1. Push changes to GitHub
2. Render will automatically redeploy
3. Check deployment logs for issues

### Frontend Updates
1. Push changes to GitHub
2. Vercel will automatically redeploy
3. Check deployment status

### Database Migrations
1. Add new migration files
2. Push to GitHub
3. Render will run migrations on next deployment

## 🚨 Security Considerations

### Production Security Checklist
- [ ] Change all default passwords
- [ ] Use strong JWT secrets
- [ ] Enable HTTPS (automatic on Render/Vercel)
- [ ] Configure proper CORS settings
- [ ] Set up monitoring and alerts
- [ ] Regular security updates

### Environment Variables Security
- Never commit secrets to Git
- Use Render/Vercel environment variable management
- Rotate secrets regularly
- Use different secrets for different environments

## 📞 Support

### Render Support
- [Render Documentation](https://render.com/docs)
- [Render Community](https://community.render.com)

### Vercel Support
- [Vercel Documentation](https://vercel.com/docs)
- [Vercel Community](https://github.com/vercel/vercel/discussions)

### FundHub Support
- Check GitHub issues
- Review deployment logs
- Test locally first

## 🎉 Success!

Once deployed, your FundHub platform will be available at:
- **Frontend**: `https://your-app-name.vercel.app`
- **Backend API**: `https://your-backend-name.onrender.com`
- **API Docs**: `https://your-backend-name.onrender.com/api/docs`

Your decentralized crowdfunding platform is now live! 🚀
