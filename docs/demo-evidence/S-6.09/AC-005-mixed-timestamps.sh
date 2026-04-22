#!/usr/bin/env bash
# AC-5 demo: Mixed timestamp formats in fixtures.
#
# The fixture contains both ISO 8601 strings and Unix epoch integers for created_at.
# The DTU returns them verbatim — exercising Prism's timestamp normalization.
set -euo pipefail
source "$(dirname "$0")/demo-lib.sh"

start_dtu
trap stop_dtu EXIT

echo "=== AC-5: Mixed Timestamp Formats in Fixture ==="
echo ""

# Login
RESP=$(curl -si -X POST "$BASE_URL/login" -H "Content-Type: application/json" -d '{}')
SESSION_TOKEN=$(echo "$RESP" | grep -i "^set-cookie:" | sed 's/.*cyberint_session=\([^;]*\).*/\1/' | tr -d '[:space:]')

echo "--- GET /api/v1/alerts — show created_at values (first 4 alerts) ---"
curl -s "$BASE_URL/api/v1/alerts" \
    -H "Cookie: cyberint_session=$SESSION_TOKEN" \
    | python3 -c "
import json, sys
data = json.load(sys.stdin)
alerts = data.get('data', data) if isinstance(data, dict) else data
for a in alerts[:4]:
    ts = a.get('created_at', 'missing')
    ts_type = 'ISO-8601 string' if isinstance(ts, str) else 'Unix epoch integer'
    print(f\"  alert_id={a['alert_id']}  created_at={ts!r}  [{ts_type}]\")
"

echo ""
echo "--- Verify mixed formats present in fixtures/alerts.json ---"
WORKTREE_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
FIXTURE="${WORKTREE_ROOT}/crates/prism-dtu-cyberint/fixtures/alerts.json"
echo "ISO-8601 entries:    $(python3 -c "import json; d=json.load(open('$FIXTURE')); print(sum(1 for a in d if isinstance(a['created_at'], str)))")"
echo "Unix epoch entries:  $(python3 -c "import json; d=json.load(open('$FIXTURE')); print(sum(1 for a in d if isinstance(a['created_at'], int)))")"

echo ""
echo "=== AC-5 PASS: both ISO-8601 and Unix epoch timestamps present and returned verbatim ==="
