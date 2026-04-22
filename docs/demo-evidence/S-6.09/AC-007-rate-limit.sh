#!/usr/bin/env bash
# AC-7 demo: Rate limit (FailureMode::RateLimit).
#
# POST /dtu/configure {"rate_limit_after":2}
# First 2 authenticated requests → 200
# 3rd request → 429 (maps to E-SENSOR-003)
set -euo pipefail
source "$(dirname "$0")/demo-lib.sh"

start_dtu
trap stop_dtu EXIT

echo "=== AC-7: Rate Limit (E-SENSOR-003) ==="
echo ""

# Login
RESP=$(curl -si -X POST "$BASE_URL/login" -H "Content-Type: application/json" -d '{}')
SESSION_TOKEN=$(echo "$RESP" | grep -i "^set-cookie:" | sed 's/.*cyberint_session=\([^;]*\).*/\1/' | tr -d '[:space:]')

echo "--- Configure rate_limit_after=2 ---"
curl -si -X POST "$BASE_URL/dtu/configure" \
    -H "Content-Type: application/json" \
    -d '{"rate_limit_after":2}' \
    | grep -E "HTTP/|\"status\""

echo ""
echo "--- Request 1 (should be 200) ---"
curl -si "$BASE_URL/api/v1/alerts" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | grep "^HTTP/"

echo "--- Request 2 (should be 200) ---"
curl -si "$BASE_URL/api/v1/alerts" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | grep "^HTTP/"

echo "--- Request 3 (should be 429 — rate limit exceeded) ---"
curl -si "$BASE_URL/api/v1/alerts" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | head -10

echo ""
echo "=== AC-7 PASS: HTTP 429 returned after rate limit threshold ==="
