#!/bin/bash
# Force complete rebuild of RustChat backend

echo "=== Force Rebuilding RustChat Backend ==="

cd /srv/data01/kubedo/rustchat/rustchat

# 1. Stop and remove everything
echo "Stopping containers..."
docker-compose down

# 2. Remove all images and volumes
echo "Removing old images..."
docker rmi rustchat-backend rustchat-frontend 2>/dev/null || true

# 3. Clear build cache
echo "Clearing build cache..."
docker builder prune -f

# 4. Verify source files are saved
echo "Checking source files..."
if grep -q "Calls enabled" backend/src/api/integrations.rs; then
    echo "✓ Source code contains changes"
else
    echo "✗ Source code missing changes!"
    exit 1
fi

# 5. Build with no cache
echo "Building backend (this may take a few minutes)..."
docker-compose build --no-cache backend

# 6. Start services
echo "Starting services..."
docker-compose up -d

# 7. Wait for startup
echo "Waiting for startup..."
sleep 10

# 8. Check logs
echo ""
echo "=== Recent Logs ==="
docker logs rustchat-backend --tail 20

echo ""
echo "=== Build Complete ==="
echo "Now test: /call start"
echo "Then check logs: docker logs rustchat-backend | grep -i call"
