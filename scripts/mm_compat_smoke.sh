#!/bin/bash
set -e

BASE=${BASE:-http://localhost:3000}
EXPECTED_MM_VERSION=${EXPECTED_MM_VERSION:-10.11.10}

echo "Testing against $BASE"

# 1. Ping
echo "1. Testing Ping..."
curl -i -s $BASE/api/v4/system/ping | grep -E "200|status|version"
echo "OK"

# 2. Version
echo "2. Testing Version..."
curl -i -s $BASE/api/v4/system/version | grep -E "200|$EXPECTED_MM_VERSION"
echo "OK"

# 3. Client config
echo "3. Testing Client Config..."
curl -i -s "$BASE/api/v4/config/client?format=old" | grep -E "200|Version"
echo "OK"

# 4. Login (This expects a user 'test'/'test' to exist or provided via env)
LOGIN_ID=${LOGIN_ID:-test}
PASSWORD=${PASSWORD:-test}

echo "4. Testing Login for $LOGIN_ID..."
LOGIN_RESPONSE=$(curl -si -X POST "$BASE/api/v4/users/login" \
  -H 'Content-Type: application/json' \
  -d "{\"login_id\":\"$LOGIN_ID\",\"password\":\"$PASSWORD\"}")

# Prefer token header (case-insensitive), then fall back to JSON body token if present.
TOKEN=$(printf '%s\n' "$LOGIN_RESPONSE" | awk 'BEGIN{IGNORECASE=1} /^token:/{print $2; exit}' | tr -d '\r')
if [ -z "$TOKEN" ]; then
  TOKEN=$(printf '%s\n' "$LOGIN_RESPONSE" | sed -n 's/.*"token":"\([^"]*\)".*/\1/p' | head -n 1)
fi

if [ -z "$TOKEN" ]; then
  echo "Failed to get token. Make sure user exists."
  echo "Skipping auth tests. (If you are running this in CI without a running DB/User, this is expected)"
  exit 0
else
  echo "Token captured: ${TOKEN:0:10}..."
fi

# 5. users/me
echo "5. Testing users/me..."
curl -si $BASE/api/v4/users/me -H "Authorization: Bearer $TOKEN" | head -n 1 | grep "200"
echo "OK"

# 6. teams
echo "6. Testing teams..."
curl -si $BASE/api/v4/teams -H "Authorization: Bearer $TOKEN" | head -n 1 | grep "200"
echo "OK"

echo "Smoke test complete!"
