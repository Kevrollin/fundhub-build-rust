# FundHub Deployment Guide - Step by Step

This guide will walk you through deploying both the backend (Rust) and frontend (React) applications to production.

## Prerequisites

- GitHub repository with your code
- Render account (for backend)
- Vercel account (for frontend)
- PostgreSQL database (Render provides this)
- Redis database (Render provides this)

## Part 1: Backend Deployment (Render)

### Step 1: Prepare Your Repository

1. **Ensure your code is pushed to GitHub:**
   ```bash
   git add .
   git commit -m "Prepare for deployment"
   git push origin main
   ```

2. **Verify these files exist in your repository:**
   - `render.yaml` (updated configuration)
   - `Dockerfile.render` (optimized for Render)
   - `init-db.sh` (database initialization script)
   - `migrations/` folder with all migration files

### Step 2: Create Render Account and Connect Repository

1. **Go to [Render.com](https://render.com) and sign up**
2. **Connect your GitHub account**
3. **Import your repository**

### Step 3: Create Database Services

1. **Create PostgreSQL Database:**
   - Go to "New" → "PostgreSQL"
   - Name: `fundhub-db`
   - Plan: Choose appropriate plan (Free tier available)
   - Region: Choose closest to your users
   - Click "Create Database"
   - **Save the connection string** - you'll need this for environment variables

2. **Create Redis Database:**
   - Go to "New" → "Redis"
   - Name: `fundhub-redis`
   - Plan: Choose appropriate plan (Free tier available)
   - Region: Same as PostgreSQL
   - Click "Create Redis"
   - **Save the connection string**

### Step 4: Deploy Backend Service

1. **Create Web Service:**
   - Go to "New" → "Web Service"
   - Connect your GitHub repository
   - Choose the repository and branch (usually `main`)

2. **Configure Service Settings:**
   - **Name:** `fundhub-backend`
   - **Environment:** `Docker`
   - **Dockerfile Path:** `./Dockerfile.render`
   - **Plan:** Choose appropriate plan (Free tier available)
   - **Region:** Same as your databases

3. **Set Environment Variables:**
   Click on "Environment" tab and add these variables:

   **Required Variables:**
   ```
   DATABASE_URL = [Your PostgreSQL connection string from Step 3]
   REDIS_URL = [Your Redis connection string from Step 3]
   JWT_SECRET = [Generate a strong random string - 32+ characters]
   STELLAR_NETWORK = testnet
   STELLAR_HORIZON_URL = https://horizon-testnet.stellar.org
   SOROBAN_RPC_URL = https://soroban-testnet.stellar.org
   PLATFORM_WALLET_PUBLIC_KEY = [Your Stellar wallet public key]
   RUST_LOG = info
   CORS_ORIGINS = *
   ```

   **Optional Variables (if needed):**
   ```
   MINIO_ENDPOINT = [If using file storage]
   MINIO_ACCESS_KEY = [If using file storage]
   MINIO_SECRET_KEY = [If using file storage]
   MINIO_BUCKET = [If using file storage]
   SMTP_HOST = [If using email notifications]
   SMTP_PORT = 587
   SMTP_USERNAME = [If using email notifications]
   SMTP_PASSWORD = [If using email notifications]
   SMTP_FROM = [If using email notifications]
   ```

4. **Deploy:**
   - Click "Create Web Service"
   - Render will automatically build and deploy your application
   - **Save the service URL** (e.g., `https://fundhub-backend.onrender.com`)

### Step 5: Verify Backend Deployment

1. **Check deployment logs** in Render dashboard
2. **Test the health endpoint:** `https://your-backend-url.onrender.com/health`
3. **Verify database connection** by checking logs for successful migration

## Part 2: Frontend Deployment (Vercel)

### Step 1: Prepare Frontend for Deployment

1. **Navigate to frontend directory:**
   ```bash
   cd fundbloom-nexus
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Test build locally:**
   ```bash
   npm run build
   ```

### Step 2: Create Vercel Account and Connect Repository

1. **Go to [Vercel.com](https://vercel.com) and sign up**
2. **Connect your GitHub account**
3. **Import your repository**

### Step 3: Configure Frontend Deployment

1. **Create New Project:**
   - Click "New Project"
   - Select your repository
   - **Root Directory:** `fundbloom-nexus`
   - **Framework Preset:** Vite
   - Click "Deploy"

2. **Set Environment Variables:**
   Go to Project Settings → Environment Variables and add:

   **Required Variables:**
   ```
   VITE_API_BASE_URL = https://your-backend-url.onrender.com/api
   VITE_STELLAR_NETWORK = testnet
   VITE_HORIZON_URL = https://horizon-testnet.stellar.org
   VITE_SOROBAN_RPC_URL = https://soroban-testnet.stellar.org
   ```

   **Optional Variables:**
   ```
   VITE_PLATFORM_NAME = FundHub
   VITE_PLATFORM_DESCRIPTION = Decentralized funding platform
   VITE_ENABLE_ANALYTICS = true
   VITE_ENABLE_NOTIFICATIONS = true
   VITE_ENABLE_WALLET_CONNECTION = true
   ```

3. **Redeploy with Environment Variables:**
   - Go to "Deployments" tab
   - Click "Redeploy" on the latest deployment
   - Or push a new commit to trigger automatic deployment

### Step 4: Verify Frontend Deployment

1. **Check deployment status** in Vercel dashboard
2. **Visit your frontend URL** (provided by Vercel)
3. **Test wallet connection** and API calls
4. **Verify all features work** (registration, login, wallet connection)

## Part 3: Post-Deployment Configuration

### Step 1: Update CORS Settings

1. **In your Render backend service:**
   - Go to Environment Variables
   - Update `CORS_ORIGINS` to include your Vercel frontend URL:
   ```
   CORS_ORIGINS = https://your-frontend-url.vercel.app,https://your-backend-url.onrender.com
   ```
   - Redeploy the service

### Step 2: Configure Custom Domains (Optional)

1. **For Backend (Render):**
   - Go to your service settings
   - Add custom domain if desired
   - Update DNS records as instructed

2. **For Frontend (Vercel):**
   - Go to project settings
   - Add custom domain
   - Update DNS records as instructed

### Step 3: Set Up Monitoring

1. **Backend Monitoring:**
   - Render provides basic monitoring
   - Check logs regularly for errors
   - Set up alerts if needed

2. **Frontend Monitoring:**
   - Vercel provides analytics
   - Monitor performance and errors
   - Set up alerts for critical issues

## Part 4: Testing Your Deployment

### Step 1: Backend Testing

1. **Health Check:**
   ```bash
   curl https://your-backend-url.onrender.com/health
   ```

2. **API Endpoints:**
   ```bash
   # Test registration
   curl -X POST https://your-backend-url.onrender.com/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"email":"test@example.com","password":"password123","name":"Test User"}'
   
   # Test login
   curl -X POST https://your-backend-url.onrender.com/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"test@example.com","password":"password123"}'
   ```

### Step 2: Frontend Testing

1. **Visit your frontend URL**
2. **Test user registration**
3. **Test user login**
4. **Test wallet connection**
5. **Test all major features**

## Part 5: Troubleshooting

### Common Backend Issues

1. **Database Connection Failed:**
   - Check `DATABASE_URL` environment variable
   - Verify database is running in Render
   - Check database credentials

2. **Build Failures:**
   - Check Docker logs in Render
   - Verify all dependencies are in `Cargo.toml`
   - Ensure `Dockerfile.render` is correct

3. **CORS Errors:**
   - Update `CORS_ORIGINS` environment variable
   - Include both frontend and backend URLs
   - Redeploy backend service

### Common Frontend Issues

1. **API Connection Failed:**
   - Check `VITE_API_BASE_URL` environment variable
   - Verify backend is running and accessible
   - Check CORS settings on backend

2. **Build Failures:**
   - Check build logs in Vercel
   - Verify all dependencies are in `package.json`
   - Check for TypeScript errors

3. **Environment Variables Not Working:**
   - Ensure variables start with `VITE_`
   - Redeploy after adding new variables
   - Check variable names match exactly

## Part 6: Production Checklist

- [ ] Backend deployed and accessible
- [ ] Database connected and migrations run
- [ ] Frontend deployed and accessible
- [ ] Environment variables configured
- [ ] CORS settings updated
- [ ] Health checks passing
- [ ] User registration working
- [ ] User login working
- [ ] Wallet connection working
- [ ] All major features tested
- [ ] Custom domains configured (if desired)
- [ ] Monitoring set up
- [ ] SSL certificates working (automatic with Render/Vercel)

## Support and Resources

- **Render Documentation:** https://render.com/docs
- **Vercel Documentation:** https://vercel.com/docs
- **Stellar Documentation:** https://developers.stellar.org
- **Project Repository:** [Your GitHub repository URL]

## Next Steps

1. **Monitor your applications** for the first few days
2. **Set up automated backups** for your database
3. **Configure monitoring and alerting**
4. **Plan for scaling** as your user base grows
5. **Consider setting up staging environment** for testing

---

**Important Notes:**
- Free tiers have limitations (sleeping services, limited resources)
- Consider upgrading to paid plans for production use
- Always test thoroughly before going live
- Keep your environment variables secure
- Regularly update dependencies for security
