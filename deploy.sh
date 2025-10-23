#!/bin/bash

# FundHub Deployment Script
# This script helps you deploy FundHub to Render and Vercel

set -e

echo "🚀 FundHub Deployment Script"
echo "=============================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -f "fundbloom-nexus/package.json" ]; then
    echo "❌ Error: Please run this script from the FundHub root directory"
    exit 1
fi

echo "✅ Found FundHub project files"

# Check for required files
echo "🔍 Checking deployment files..."

required_files=(
    "render.yaml"
    "init-db.sh"
    "fundbloom-nexus/vercel.json"
    "migrations/"
)

for file in "${required_files[@]}"; do
    if [ ! -e "$file" ]; then
        echo "❌ Missing required file: $file"
        exit 1
    fi
done

echo "✅ All required deployment files found"

# Make scripts executable
echo "🔧 Making scripts executable..."
chmod +x init-db.sh
chmod +x setup-production-db.sh

echo "✅ Scripts are executable"

# Check Git status
echo "📋 Checking Git status..."
if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  Warning: You have uncommitted changes"
    echo "   Consider committing your changes before deployment"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Deployment cancelled"
        exit 1
    fi
fi

echo "✅ Git status checked"

# Display next steps
echo ""
echo "🎯 Next Steps:"
echo "=============="
echo ""
echo "1. Backend Deployment (Render):"
echo "   - Go to https://dashboard.render.com"
echo "   - Create new Web Service"
echo "   - Connect your GitHub repository"
echo "   - Use the configuration from render.yaml"
echo "   - Set up PostgreSQL and Redis services"
echo "   - Configure environment variables (see render-env.md)"
echo ""
echo "2. Frontend Deployment (Vercel):"
echo "   - Go to https://vercel.com/dashboard"
echo "   - Create new project"
echo "   - Import your GitHub repository"
echo "   - Set root directory to 'fundbloom-nexus'"
echo "   - Configure environment variables (see vercel-env.md)"
echo ""
echo "3. Connect Services:"
echo "   - Update CORS settings in backend"
echo "   - Update API URL in frontend"
echo "   - Redeploy both services"
echo ""
echo "📖 For detailed instructions, see DEPLOYMENT_GUIDE.md"
echo ""
echo "🔗 Useful Links:"
echo "   - Render Dashboard: https://dashboard.render.com"
echo "   - Vercel Dashboard: https://vercel.com/dashboard"
echo "   - Stellar Testnet: https://horizon-testnet.stellar.org"
echo ""
echo "✅ Deployment preparation complete!"
echo "   Follow the steps above to deploy your FundHub platform."
