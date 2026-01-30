#!/bin/bash

echo "Setting up Track Credentials development environment..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✓ Rust found"

# Add wasm32 target
echo "Adding wasm32-unknown-unknown target..."
rustup target add wasm32-unknown-unknown

# Install Trunk
if ! command -v trunk &> /dev/null; then
    echo "Installing Trunk..."
    cargo install --locked trunk
else
    echo "✓ Trunk already installed"
fi

# Install wasm-bindgen-cli (optional but recommended)
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
else
    echo "✓ wasm-bindgen-cli already installed"
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "To build and run the application:"
echo "  1. Build frontend:  cd frontend && trunk build"
echo "  2. Run backend:     cd backend && cargo run"
echo "  3. Open browser:    http://localhost:8000"
