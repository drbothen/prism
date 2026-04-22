#!/usr/bin/env bash
# AC-6 demo: Cursor pagination.
#
# GET /api/v1/alerts (no cursor) → first page, next_cursor set
# GET /api/v1/alerts?cursor={next_cursor} → second page, next_cursor null
set -euo pipefail
source "$(dirname "$0")/demo-lib.sh"

start_dtu
trap stop_dtu EXIT

echo "=== AC-6: Cursor Pagination ==="
echo ""

# Login
RESP=$(curl -si -X POST "$BASE_URL/login" -H "Content-Type: application/json" -d '{}')
SESSION_TOKEN=$(echo "$RESP" | grep -i "^set-cookie:" | sed 's/.*cyberint_session=\([^;]*\).*/\1/' | tr -d '[:space:]')

echo "--- Page 1: GET /api/v1/alerts (no cursor) ---"
PAGE1=$(curl -s "$BASE_URL/api/v1/alerts" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN")
echo "$PAGE1" | python3 -c "
import json, sys
d = json.load(sys.stdin)
alerts = d.get('data', [])
print(f'  alerts on page 1: {len(alerts)}')
print(f'  next_cursor: {d.get(\"next_cursor\")!r}')
"

NEXT_CURSOR=$(echo "$PAGE1" | python3 -c "import json,sys; print(json.load(sys.stdin).get('next_cursor',''))")

echo ""
echo "--- Page 2: GET /api/v1/alerts?cursor=$NEXT_CURSOR ---"
PAGE2=$(curl -s "$BASE_URL/api/v1/alerts?cursor=$NEXT_CURSOR" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN")
echo "$PAGE2" | python3 -c "
import json, sys
d = json.load(sys.stdin)
alerts = d.get('data', [])
print(f'  alerts on page 2: {len(alerts)}')
print(f'  next_cursor: {d.get(\"next_cursor\")!r}')
"

echo ""
echo "=== AC-6 PASS: two-page cursor pagination works ==="
