#!/bin/bash
# EMERGENCY: Force complete rebuild with all caches cleared

cd /srv/data01/kubedo/rustchat/rustchat

echo "Step 1: Stopping all containers..."
docker-compose down 2>/dev/null

echo "Step 2: Removing ALL RustChat images..."
docker images | grep rustchat | awk '{print $3}' | xargs docker rmi -f 2>/dev/null

echo "Step 3: Clearing ALL Docker build cache..."
docker system prune -af --volumes

echo "Step 4: Verifying source code has changes..."
if ! grep -q "Calls enabled" backend/src/api/integrations.rs; then
    echo "ERROR: Source code doesn't have the changes!"
    exit 1
fi
echo "✓ Source code verified"

echo "Step 5: Building backend (5-10 minutes)..."
docker-compose build --no-cache backend

echo "Step 6: Starting containers..."
docker-compose up -d

echo "Step 7: Waiting for startup..."
sleep 15

echo ""
echo "=== COMPLETE ==="
echo "Container started. Now test /call command and check:"
echo "docker logs rustchat-backend | grep -i 'calls enabled'"
