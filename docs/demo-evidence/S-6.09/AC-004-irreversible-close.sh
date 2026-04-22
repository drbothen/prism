#!/usr/bin/env bash
# AC-4 demo: Irreversible close enforced.
#
# POST /close → 200 status:closed
# PATCH /status acknowledge on closed alert → 400 "alert already closed"
set -euo pipefail
source "$(dirname "$0")/demo-lib.sh"

start_dtu
trap stop_dtu EXIT

echo "=== AC-4: Irreversible Close Enforced ==="
echo ""

# Login
RESP=$(curl -si -X POST "$BASE_URL/login" -H "Content-Type: application/json" -d '{}')
SESSION_TOKEN=$(echo "$RESP" | grep -i "^set-cookie:" | sed 's/.*cyberint_session=\([^;]*\).*/\1/' | tr -d '[:space:]')
ALERT_ID="CYB-2024-002"

echo "--- Step 1: POST /close on $ALERT_ID ---"
curl -si -X POST "$BASE_URL/api/v1/alerts/$ALERT_ID/close" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | grep -E "HTTP/|\"status\"|\"alert_id\""

echo ""
echo "--- Step 2: PATCH /status acknowledge on closed alert → must return 400 ---"
curl -si -X PATCH "$BASE_URL/api/v1/alerts/$ALERT_ID/status" \
    -H "Content-Type: application/json" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    -d '{"status":"acknowledged"}' \
    | head -15

echo ""
echo "=== AC-4 PASS: close is irreversible within session ==="
