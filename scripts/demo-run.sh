#!/usr/bin/env bash
# scripts/demo-run.sh — Start the Prism DTU demo server + wait for it to be ready.
#
# USAGE
#   bash scripts/demo-run.sh [--config-dir DIR]
#
# WHAT IT DOES
#   1. Starts prism-dtu-demo-server in the background
#   2. Polls .prism-dtu-demo-server.urls.json for up to 30s (EC-004)
#   3. Prints the DTU clone URLs (CrowdStrike, Armis, Claroty, Cyberint)
#   4. Prints the command to start prism-bin
#
# OUTPUTS
#   A urls.json sidecar at ${DEMO_CONFIG_DIR}/run/.prism-dtu-demo-server.urls.json
#   A PID file at ${DEMO_CONFIG_DIR}/run/.prism-dtu-demo-server.pid
#
# Run demo-teardown.sh to stop the server and clean up.
#
# AC-003: DTU server starts in background; ports printed within 30s.
# AC-008: passes shellcheck with zero errors/warnings.
#
# Story: S-DEMO-003

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

DEMO_CONFIG_DIR="${HOME}/.config/prism-demo"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --config-dir)
            DEMO_CONFIG_DIR="$2"
            shift 2
            ;;
        --config-dir=*)
            DEMO_CONFIG_DIR="${1#--config-dir=}"
            shift
            ;;
        *)
            echo "Usage: $0 [--config-dir DIR]" >&2
            exit 1
            ;;
    esac
done

DEMO_RUN_DIR="${DEMO_CONFIG_DIR}/run"
DTU_BIN="${REPO_ROOT}/target/release/prism-dtu-demo-server"
DTU_CONFIG="${REPO_ROOT}/scripts/demo.toml"
URLS_FILE="${DEMO_RUN_DIR}/.prism-dtu-demo-server.urls.json"

# ---------------------------------------------------------------------------
# Verify prerequisites
# ---------------------------------------------------------------------------

if [[ ! -f "${DTU_BIN}" ]]; then
    echo "ERROR: prism-dtu-demo-server not found at ${DTU_BIN}" >&2
    echo "       Run: bash scripts/demo-setup.sh" >&2
    exit 1
fi

if [[ ! -f "${DEMO_CONFIG_DIR}/prism.toml" ]]; then
    echo "ERROR: prism.toml not found at ${DEMO_CONFIG_DIR}/prism.toml" >&2
    echo "       Run: bash scripts/demo-setup.sh" >&2
    exit 1
fi

mkdir -p "${DEMO_RUN_DIR}"

# Clean up any stale URL file from a previous run.
rm -f "${URLS_FILE}"

# ---------------------------------------------------------------------------
# Start DTU demo server in background
# ---------------------------------------------------------------------------

echo "==> Starting prism-dtu-demo-server..."

# The demo server writes .prism-dtu-demo-server.urls.json in its working
# directory. We cd into DEMO_RUN_DIR so the file lands there.
(
    cd "${DEMO_RUN_DIR}"
    "${DTU_BIN}" start --config "${DTU_CONFIG}" \
        >> "${DEMO_RUN_DIR}/dtu-server.log" 2>&1 &
    echo "$!" > "${DEMO_RUN_DIR}/.prism-dtu-demo-server.pid"
)

DTU_PID=$(cat "${DEMO_RUN_DIR}/.prism-dtu-demo-server.pid" 2>/dev/null || echo "unknown")
echo "    DTU server started (PID ${DTU_PID})"

# ---------------------------------------------------------------------------
# Poll for URLs file (EC-004: 30s timeout)
# ---------------------------------------------------------------------------

echo "==> Waiting for DTU clones to bind (up to 30s)..."

POLL_TIMEOUT=30
POLL_ELAPSED=0
while [[ ! -f "${URLS_FILE}" ]]; do
    sleep 1
    POLL_ELAPSED=$((POLL_ELAPSED + 1))
    if [[ "${POLL_ELAPSED}" -ge "${POLL_TIMEOUT}" ]]; then
        echo "ERROR: DTU server did not start within ${POLL_TIMEOUT}s." >&2
        echo "       Check ${DEMO_RUN_DIR}/dtu-server.log for details." >&2
        echo "       Common cause: port conflict — stop other services on the demo ports." >&2
        exit 1
    fi
done

echo "    URLs file ready after ${POLL_ELAPSED}s"

# ---------------------------------------------------------------------------
# Print DTU clone URLs
# ---------------------------------------------------------------------------

echo ""
echo "==> DTU clone URLs:"
# Parse JSON with python3 (available on all macOS/Linux demo targets).
if command -v python3 &>/dev/null; then
    python3 - "${URLS_FILE}" << 'PYEOF'
import json, sys
with open(sys.argv[1]) as f:
    urls = json.load(f)
for name, url in sorted(urls.items()):
    print(f"    {name:15s}: {url}")
PYEOF
else
    cat "${URLS_FILE}"
fi
echo ""

# ---------------------------------------------------------------------------
# Print prism-bin start command
# ---------------------------------------------------------------------------

PRISM_BIN="${REPO_ROOT}/target/release/prism"

echo "==> To start prism, run in a new terminal:"
echo ""
echo "    ${PRISM_BIN} --config-dir ${DEMO_CONFIG_DIR} start"
echo ""
echo "    Then add prism to Claude Code (see docs/DEMO-RUNBOOK.md §3)."
echo ""
echo "==> DTU server log: ${DEMO_RUN_DIR}/dtu-server.log"
echo "    To stop everything: bash scripts/demo-teardown.sh"
echo ""
