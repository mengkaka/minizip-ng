#!/bin/bash
# Script to build minizip for M4 Mac (Apple Silicon)
set -e

echo "Building minizip for M4 Mac..."
cd "$(dirname "$0")/../../rust"

# Check if rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust from https://sh.rustup.rs"
    exit 1
fi

cargo build --release

echo "Build successful!"
echo "The binary is located at: $(pwd)/target/release/minizip"
echo "You can copy it to your preferred location and run it."
