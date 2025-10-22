#!/bin/bash

# Full development script for FundHub
# Runs both frontend (Vite) and backend (Rust) with auto-reload

echo "🚀 Starting FundHub Full Development Environment"
echo ""

# Function to cleanup background processes on exit
cleanup() {
    echo ""
    echo "🛑 Shutting down development servers..."
    kill $FRONTEND_PID $BACKEND_PID 2>/dev/null
    exit
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Start backend in background
echo "🔧 Starting Backend Server (Rust + Axum)..."
cd /home/dev-mk/Desktop/Projects/fundhub-build
./scripts/dev-backend.sh &
BACKEND_PID=$!

# Wait a moment for backend to start
sleep 3

# Start frontend in background
echo "🎨 Starting Frontend Server (Vite + React)..."
cd /home/dev-mk/Desktop/Projects/fundhub-build/fundbloom-nexus
npm run dev &
FRONTEND_PID=$!

echo ""
echo "✅ Development servers started!"
echo "🌐 Frontend: http://localhost:8080"
echo "🔧 Backend: http://localhost:3000"
echo "📚 API Docs: http://localhost:3000/api/docs"
echo ""
echo "Press Ctrl+C to stop all servers"

# Wait for both processes
wait $FRONTEND_PID $BACKEND_PID
