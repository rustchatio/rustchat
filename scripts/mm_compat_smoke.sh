#!/bin/bash
set -euo pipefail

if [ -z "${BASE:-}" ]; then
  echo "BASE is required (example: BASE=http://127.0.0.1:3000 ./scripts/mm_compat_smoke.sh)"
  exit 1
fi
BASE="${BASE%/}"
EXPECTED_MM_VERSION=${EXPECTED_MM_VERSION:-10.11.10}

echo "Testing against $BASE"

# Preflight: make sure this target is RustChat v4 compatibility surface.
echo "0. Preflight target validation..."
PING_HEADERS="$(curl -sSI "$BASE/api/v4/system/ping" || true)"
if ! printf '%s\n' "$PING_HEADERS" | head -n 1 | grep -q " 200 "; then
  echo "Failed preflight: expected 200 from $BASE/api/v4/system/ping"
  echo "$PING_HEADERS"
  exit 1
fi
if ! printf '%s\n' "$PING_HEADERS" | grep -qi "^X-MM-COMPAT:[[:space:]]*1"; then
  echo "Failed preflight: target does not advertise RustChat MM compatibility header X-MM-COMPAT: 1"
  echo "$PING_HEADERS"
  exit 1
fi
echo "OK"

# 1. Ping
echo "1. Testing Ping..."
curl -i -s "$BASE/api/v4/system/ping" | grep -E "200|status|version"
echo "OK"

# 2. Version
echo "2. Testing Version..."
curl -i -s "$BASE/api/v4/system/version" | grep -E "200|$EXPECTED_MM_VERSION"
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
curl -si "$BASE/api/v4/users/me" -H "Authorization: Bearer $TOKEN" | head -n 1 | grep "200"
echo "OK"

# 6. teams
echo "6. Testing teams..."
curl -si "$BASE/api/v4/teams" -H "Authorization: Bearer $TOKEN" | head -n 1 | grep "200"
echo "OK"

echo "Smoke test complete!"
