#!/usr/bin/env bash
# AC-3 demo: Alert status transition persists.
#
# PATCH /api/v1/alerts/{id}/status {"status":"acknowledged"} → 200
# GET  /api/v1/alerts/{id} → status: "acknowledged"
set -euo pipefail
source "$(dirname "$0")/demo-lib.sh"

start_dtu
trap stop_dtu EXIT

echo "=== AC-3: Alert Status Transition (Stateful) ==="
echo ""

# Login
RESP=$(curl -si -X POST "$BASE_URL/login" -H "Content-Type: application/json" -d '{}')
SESSION_TOKEN=$(echo "$RESP" | grep -i "^set-cookie:" | sed 's/.*cyberint_session=\([^;]*\).*/\1/' | tr -d '[:space:]')
ALERT_ID="CYB-2024-001"

echo "--- Step 1: GET initial status of $ALERT_ID ---"
curl -si "$BASE_URL/api/v1/alerts/$ALERT_ID" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | grep -E "HTTP/|\"status\""

echo ""
echo "--- Step 2: PATCH status to 'acknowledged' ---"
curl -si -X PATCH "$BASE_URL/api/v1/alerts/$ALERT_ID/status" \
    -H "Content-Type: application/json" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    -d '{"status":"acknowledged"}' \
    | grep -E "HTTP/|\"status\""

echo ""
echo "--- Step 3: GET status after PATCH (must show acknowledged) ---"
curl -si "$BASE_URL/api/v1/alerts/$ALERT_ID" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | grep -E "HTTP/|\"status\""

echo ""
echo "=== AC-3 PASS: stateful status transition persists ==="
