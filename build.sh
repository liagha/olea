#!/bin/bash
set -e

echo "Building custom Linux distribution..."

# Ensure src directory exists
mkdir -p src

# Copy the Rust init program from src directory if it exists, otherwise create it
if [ -f src/init.rs ]; then
    echo "Using existing init.rs from src directory."
else
    echo "init.rs not found in src directory..."
fi

# Build kernel image
echo "Building Linux kernel..."
docker build -t kernel-build -f Dockerfile.kernel .

# Build init program
echo "Building Rust init program..."
docker build -t init-build -f Dockerfile.init .

# Build zsh and uutils
echo "Building zsh and uutils coreutils..."
docker build -t utils-build -f Dockerfile.utils .

# Build final image
echo "Building final distribution image..."
docker build -t my-linux-distro -f Dockerfile.main .

echo "Build complete! Run the distribution with:"
echo "docker run --rm -it my-linux-distro"