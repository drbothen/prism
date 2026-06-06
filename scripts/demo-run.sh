#!/usr/bin/env bash
# scripts/demo-run.sh — Start the Prism DTU demo server + wait for it to be ready.
#
# USAGE
#   bash scripts/demo-run.sh [--config-dir DIR]
#
# WHAT IT DOES
#   1. Starts prism-dtu-demo-server in the background on four ephemeral ports
#   2. Polls .prism-dtu-demo-server.urls.json for up to 30s (EC-004)
#   3. Generates per-org overlay TOMLs with the ephemeral base_url for each sensor
#      (written AFTER ports are known — see §Overlay generation below)
#   4. Prints the DTU clone URLs (CrowdStrike, Armis, Claroty, Cyberint)
#   5. Prints the command to start prism-bin
#
# OVERLAY GENERATION (F-HIGH-201 fix)
#   DTU ports are ephemeral (port = 0 in demo.toml); only known after the DTU server
#   starts and writes its urls.json sidecar. This script reads that sidecar and writes
#   per-org sensor overlay files at:
#     ${DEMO_CONFIG_DIR}/specs/customers/demo-org/<sensor>.sensor.toml
#   Each overlay contains:
#     extends     = "<sensor>"
#     instance_id = "<sensor>@demo-org"
#     base_url    = "http://127.0.0.1:<port>"
#   This is the same BC-2.06.012 / ADR-029 hybrid overlay format used by the E2E test
#   harness (crates/prism-bin/tests/helpers/mod.rs write_sensor_overlay).
#   prism's spec loader derives customers_dir = spec_dir + "/customers" at boot time
#   (crates/prism-bin/src/boot.rs step4c).
#
# OUTPUTS
#   A urls.json sidecar at ${DEMO_CONFIG_DIR}/run/.prism-dtu-demo-server.urls.json
#   A PID file at ${DEMO_CONFIG_DIR}/run/.prism-dtu-demo-server.pid
#   Per-sensor overlays at ${DEMO_CONFIG_DIR}/specs/customers/demo-org/<sensor>.sensor.toml
#
# Run demo-teardown.sh to stop the server and clean up.
#
# AC-003: DTU server starts in background; ports printed within 30s.
# AC-009: demo queries return data rows (requires overlay generation to wire base_url).
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
# Generate per-org overlay TOMLs (F-HIGH-201)
#
# DTU ports are ephemeral — only known now (after urls.json is ready).
# prism's spec loader reads customers_dir = spec_dir + "/customers" at boot step 4c
# (crates/prism-bin/src/boot.rs). Each sensor overlay sets base_url to the DTU port.
# Overlay format: BC-2.06.012 / ADR-029 scalar-only hybrid instance overlay.
# ---------------------------------------------------------------------------

DEMO_SPECS_DIR="${DEMO_CONFIG_DIR}/specs"
DEMO_CUSTOMERS_DIR="${DEMO_SPECS_DIR}/customers"
DEMO_ORG_SLUG="demo-org"
DEMO_ORG_OVERLAY_DIR="${DEMO_CUSTOMERS_DIR}/${DEMO_ORG_SLUG}"

echo "==> Generating per-org sensor overlays (base_url → DTU ports)..."

mkdir -p "${DEMO_ORG_OVERLAY_DIR}"

# Parse urls.json and write one overlay per sensor.
# urls.json format: {"crowdstrike": "http://127.0.0.1:PORT", "armis": "...", ...}
python3 - "${URLS_FILE}" "${DEMO_ORG_OVERLAY_DIR}" "${DEMO_ORG_SLUG}" << 'PYEOF'
import json, sys, os

urls_file   = sys.argv[1]
overlay_dir = sys.argv[2]
org_slug    = sys.argv[3]

with open(urls_file) as f:
    urls = json.load(f)

for sensor_id, base_url in urls.items():
    overlay_path = os.path.join(overlay_dir, f"{sensor_id}.sensor.toml")
    content = (
        f"# Per-org overlay for {sensor_id} sensor — {org_slug} (DTU demo)\n"
        f"# BC-2.06.012 / ADR-029: scalar-only overlay; no schema fields.\n"
        f"# Generated by demo-run.sh after DTU server starts (ephemeral port).\n"
        f"extends     = \"{sensor_id}\"\n"
        f"instance_id = \"{sensor_id}@{org_slug}\"\n"
        f"base_url    = \"{base_url}\"\n"
    )
    with open(overlay_path, "w") as out:
        out.write(content)
    print(f"    Overlay written: {overlay_path}")
PYEOF

echo "    Sensor overlays ready at ${DEMO_ORG_OVERLAY_DIR}/"
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
