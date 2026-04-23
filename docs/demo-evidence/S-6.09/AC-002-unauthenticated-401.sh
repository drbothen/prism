#!/usr/bin/env bash
# AC-2 demo: Unauthenticated request returns HTTP 401.
#
# GET /api/v1/alerts without Cookie → 401 {"error":"unauthorized","code":401}
set -euo pipefail
source "$(dirname "$0")/demo-lib.sh"

start_dtu
trap stop_dtu EXIT

echo "=== AC-2: Unauthenticated Request Returns 401 ==="
echo ""
echo "--- Request: GET /api/v1/alerts (no Cookie header) ---"
curl -si "$BASE_URL/api/v1/alerts" | head -15

echo ""
echo "=== AC-2 PASS: 401 unauthorized returned ==="
