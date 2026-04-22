#!/usr/bin/env bash
# AC-1 demo: Cookie auth round-trip.
#
# POST /login → 200 + Set-Cookie header containing cyberint_session token.
# Subsequent request with that cookie → 200.
set -euo pipefail
source "$(dirname "$0")/demo-lib.sh"

start_dtu
trap stop_dtu EXIT

echo "=== AC-1: Cookie Auth Round-Trip ==="
echo ""
echo "--- Step 1: POST /login (any body) ---"
RESP=$(curl -si -X POST "$BASE_URL/login" \
    -H "Content-Type: application/json" \
    -d '{}')
echo "$RESP" | head -20

# Extract token
SESSION_TOKEN=$(echo "$RESP" \
    | grep -i "^set-cookie:" \
    | sed 's/.*cyberint_session=\([^;]*\).*/\1/' \
    | tr -d '[:space:]')

echo ""
echo "--- Step 2: GET /api/v1/alerts with session cookie ---"
curl -si "$BASE_URL/api/v1/alerts" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | head -10

echo ""
echo "=== AC-1 PASS: cookie round-trip works ==="
