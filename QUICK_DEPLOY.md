# 🚀 Quick Deploy Guide

Deploy your FundHub platform in 3 simple steps!

## Prerequisites
- GitHub repository with your code
- Render account (free)
- Vercel account (free)

## Step 1: Run Deployment Script
```bash
./deploy.sh
```

## Step 2: Deploy Backend to Render
1. Go to [Render Dashboard](https://dashboard.render.com)
2. Click "New +" → "Web Service"
3. Connect GitHub repository
4. Use these settings:
   - **Name**: `fundhub-backend`
   - **Environment**: `Rust`
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./init-db.sh && ./target/release/fundhub`
5. Add PostgreSQL and Redis services
6. Set environment variables (see `render-env.md`)
7. Deploy!

## Step 3: Deploy Frontend to Vercel
1. Go to [Vercel Dashboard](https://vercel.com/dashboard)
2. Click "New Project"
3. Import GitHub repository
4. Set root directory to `fundbloom-nexus`
5. Set environment variables (see `vercel-env.md`)
6. Deploy!

## Step 4: Connect Services
1. Update CORS in backend with your Vercel URL
2. Update API URL in frontend with your Render URL
3. Redeploy both services

## 🎉 Done!
Your FundHub platform is now live!

- **Frontend**: `https://your-app.vercel.app`
- **Backend**: `https://your-backend.onrender.com`
- **API Docs**: `https://your-backend.onrender.com/api/docs`

## 📖 Need Help?
See `DEPLOYMENT_GUIDE.md` for detailed instructions.
