#!/usr/bin/env bash
# demo-lib.sh — shared helpers for S-6.09 demo scripts.
# Source this file at the top of each per-AC demo script.
set -euo pipefail

WORKTREE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
BINARY="${WORKTREE_ROOT}/target/debug/demo_server"

# start_dtu: launch demo_server, wait for READY, export BASE_URL and SERVER_PID.
start_dtu() {
    # Build if needed (no-op if already up-to-date).
    (cd "$WORKTREE_ROOT" && cargo build -p prism-dtu-cyberint --features dtu --bin demo_server -q 2>/dev/null)

    # Start the server, capture its stdout.
    READY_FILE=$(mktemp)
    "$BINARY" >"$READY_FILE" 2>/dev/null &
    export SERVER_PID=$!

    # Wait up to 5s for READY line.
    for i in $(seq 1 50); do
        if grep -q "^READY" "$READY_FILE" 2>/dev/null; then
            break
        fi
        sleep 0.1
    done

    # Parse URL from "READY http://127.0.0.1:PORT"
    export BASE_URL
    BASE_URL=$(grep "^READY" "$READY_FILE" | head -1 | awk '{print $2}')
    rm -f "$READY_FILE"

    if [[ -z "$BASE_URL" ]]; then
        echo "ERROR: demo_server did not print READY" >&2
        exit 1
    fi
}

# stop_dtu: kill the background server.
stop_dtu() {
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
}

# login: POST /login, print Set-Cookie header, export SESSION_TOKEN.
login() {
    local resp_headers
    resp_headers=$(curl -si -X POST "$BASE_URL/login" \
        -H "Content-Type: application/json" \
        -d '{}')
    echo "$resp_headers" | grep -E "^HTTP/|^set-cookie:|^{" | head -5

    export SESSION_TOKEN
    SESSION_TOKEN=$(echo "$resp_headers" \
        | grep -i "^set-cookie:" \
        | sed 's/.*cyberint_session=\([^;]*\).*/\1/' \
        | tr -d '[:space:]')
}
