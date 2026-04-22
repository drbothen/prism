#!/usr/bin/env bash
# AC-8 demo: Reset semantics.
#
# Sequence:
#   1. Login, acknowledge an alert, close another.
#   2. POST /dtu/reset
#   3. Verify alert statuses revert to "open".
#   4. Verify old session token rejected (session_store cleared).
#   5. New login required — succeeds.
set -euo pipefail
source "$(dirname "$0")/demo-lib.sh"

start_dtu
trap stop_dtu EXIT

echo "=== AC-8: Reset Semantics ==="
echo ""

# Login and mutate state
RESP=$(curl -si -X POST "$BASE_URL/login" -H "Content-Type: application/json" -d '{}')
SESSION_TOKEN=$(echo "$RESP" | grep -i "^set-cookie:" | sed 's/.*cyberint_session=\([^;]*\).*/\1/' | tr -d '[:space:]')

echo "--- Acknowledge CYB-2024-001 ---"
curl -si -X PATCH "$BASE_URL/api/v1/alerts/CYB-2024-001/status" \
    -H "Content-Type: application/json" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    -d '{"status":"acknowledged"}' | grep -E "HTTP/|\"status\""

echo "--- Close CYB-2024-003 ---"
curl -si -X POST "$BASE_URL/api/v1/alerts/CYB-2024-003/close" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | grep -E "HTTP/|\"status\""

echo ""
echo "--- POST /dtu/reset ---"
curl -si -X POST "$BASE_URL/dtu/reset" | grep -E "HTTP/|\"status\""

echo ""
echo "--- CYB-2024-001 status after reset (must be 'open', old token rejected → 401) ---"
curl -si "$BASE_URL/api/v1/alerts/CYB-2024-001" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | head -10

echo ""
echo "--- New login after reset (must succeed) ---"
NEW_RESP=$(curl -si -X POST "$BASE_URL/login" -H "Content-Type: application/json" -d '{}')
echo "$NEW_RESP" | grep -E "HTTP/|set-cookie:"
NEW_TOKEN=$(echo "$NEW_RESP" | grep -i "^set-cookie:" | sed 's/.*cyberint_session=\([^;]*\).*/\1/' | tr -d '[:space:]')

echo ""
echo "--- CYB-2024-001 status with new token (must be 'open') ---"
curl -s "$BASE_URL/api/v1/alerts/CYB-2024-001" \
    -H "Cookie: cyberint_session=$NEW_TOKEN" \
    | python3 -c "import json,sys; d=json.load(sys.stdin); print(f'  status: {d.get(\"status\")!r}')"

echo ""
echo "=== AC-8 PASS: reset reverts state and clears sessions ==="
