#!/bin/bash
# Fast rebuild script using Docker BuildKit cache

set -e

echo "🚀 Fast rebuild starting..."

# Ensure BuildKit is enabled
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# Pull latest code
echo "📥 Pulling latest changes..."
git pull

# Build with optimized Dockerfile and cache mounts
echo "🔨 Building backend with cache..."
docker build \
    --build-arg BUILDKIT_INLINE_CACHE=1 \
    --cache-from rustchat-backend:cache \
    -f docker/backend.Dockerfile.optimized \
    -t rustchat-backend:latest \
    ./backend

# Tag for cache in next build
docker tag rustchat-backend:latest rustchat-backend:cache || true

# Restart only the backend container
echo "🔄 Restarting backend..."
docker compose up -d --no-deps --build backend

echo "✅ Done! Backend restarted."
