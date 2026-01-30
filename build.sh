#!/bin/bash
set -e

echo "🔧 Building Track Credentials..."

# Build frontend
echo "📦 Building frontend with Trunk..."
cd frontend
trunk build
cd ..

# Build backend
echo "🚀 Building backend with Cargo..."
cd backend
cargo build
cd ..

echo "✅ Build complete!"
echo ""
echo "To run the application:"
echo "  cd backend && cargo run"
echo ""
echo "The server will be available at http://localhost:8000"
