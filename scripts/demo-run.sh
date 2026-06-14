#!/usr/bin/env bash
# scripts/demo-run.sh — Start the Prism DTU demo server (multi-org) + wait for it to be ready.
#
# USAGE
#   bash scripts/demo-run.sh [--config-dir DIR]
#
# WHAT IT DOES
#   1. Starts prism-dtu-demo-server start-multi in the background (one process, all orgs)
#   2. Polls .prism-dtu-demo-server.urls-multi.json for up to 30s (EC-006)
#   3. Generates per-org overlay TOMLs with the ephemeral base_url for each sensor
#      (written AFTER ports are known — see §Overlay generation below)
#   4. Prints the DTU clone URLs ({org_slug: {sensor_id: url}})
#   5. Prints the command to start prism-bin
#
# OVERLAY GENERATION
#   DTU ports are ephemeral (port = 0 in demo.toml); only known after the DTU server
#   starts and writes its urls-multi.json nested sidecar. This script reads that sidecar
#   and writes N×M overlay files at:
#     ${DEMO_CONFIG_DIR}/specs/customers/<org_slug>/<sensor_id>.sensor.toml
#   Each overlay contains exactly:
#     extends     = "<sensor_id>"
#     instance_id = "<sensor_id>@<org_slug>"
#     base_url    = "http://127.0.0.1:<port>"
#   Per BC-2.06.013 (scalar-only overlay enforcement): no auth_type, no table fields.
#   Per BC-2.06.012: overlays are read by prism at boot step 4c (customers_dir).
#   Per BC-2.06.014: each (org_id, sensor_id) resolves to its own DTU clone endpoint.
#
# NESTED SIDECAR FORMAT (BC-2.06.017)
#   {org_slug: {sensor_id: url}} — written by start-multi to urls-multi.json.
#   Distinct from the flat {name: url} sidecar written by the `start` subcommand.
#
# OUTPUTS
#   A urls-multi.json sidecar at ${DEMO_CONFIG_DIR}/run/.prism-dtu-demo-server.urls-multi.json
#   A PID file at ${DEMO_CONFIG_DIR}/run/.prism-dtu-demo-server.pid
#   N×M overlay TOMLs at ${DEMO_CONFIG_DIR}/specs/customers/<org_slug>/<sensor_id>.sensor.toml
#
# Run demo-teardown.sh to stop the server and clean up.
#
# EC-004: prism.toml must exist before starting DTU; exits 1 if missing.
# EC-006: DTU server must write sidecar within 30s; exits 1 with actionable message.
# AC-008: passes shellcheck with zero errors/warnings.
#
# Stories: S-DEMO-003 | S-DEMO-LAUNCHER-CONSOLIDATION-001

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
# Nested sidecar written by `start-multi` (BC-2.06.017): {org_slug: {sensor_id: url}}.
# Distinct from the flat .prism-dtu-demo-server.urls.json written by `start`.
URLS_MULTI_FILE="${DEMO_RUN_DIR}/.prism-dtu-demo-server.urls-multi.json"

# ---------------------------------------------------------------------------
# Verify prerequisites
# ---------------------------------------------------------------------------

if [[ ! -f "${DTU_BIN}" ]]; then
    echo "ERROR: prism-dtu-demo-server not found at ${DTU_BIN}" >&2
    echo "       Run: bash scripts/demo-setup.sh" >&2
    exit 1
fi

# EC-004: prism.toml must exist before starting DTU server.
if [[ ! -f "${DEMO_CONFIG_DIR}/prism.toml" ]]; then
    echo "ERROR: prism.toml not found at ${DEMO_CONFIG_DIR}/prism.toml" >&2
    echo "       Run: bash scripts/demo-setup.sh" >&2
    exit 1
fi

mkdir -p "${DEMO_RUN_DIR}"

# Clean up any stale sidecar file from a previous run.
rm -f "${URLS_MULTI_FILE}"

# ---------------------------------------------------------------------------
# Start DTU demo server in background (single start-multi process — all orgs)
# ---------------------------------------------------------------------------

echo "==> Starting prism-dtu-demo-server start-multi (all orgs)..."

# The demo server writes .prism-dtu-demo-server.urls-multi.json in its working
# directory. We cd into DEMO_RUN_DIR so the file lands there.
(
    cd "${DEMO_RUN_DIR}"
    "${DTU_BIN}" start-multi --config "${DTU_CONFIG}" \
        >> "${DEMO_RUN_DIR}/dtu-server.log" 2>&1 &
    echo "$!" > "${DEMO_RUN_DIR}/.prism-dtu-demo-server.pid"
)

DTU_PID=$(cat "${DEMO_RUN_DIR}/.prism-dtu-demo-server.pid" 2>/dev/null || echo "unknown")
echo "    DTU server started (PID ${DTU_PID})"

# ---------------------------------------------------------------------------
# Poll for nested URLs file (EC-006: 30s timeout)
# ---------------------------------------------------------------------------

echo "==> Waiting for DTU clones to bind (up to 30s)..."

POLL_TIMEOUT=30
POLL_ELAPSED=0
while [[ ! -f "${URLS_MULTI_FILE}" ]]; do
    sleep 1
    POLL_ELAPSED=$((POLL_ELAPSED + 1))
    if [[ "${POLL_ELAPSED}" -ge "${POLL_TIMEOUT}" ]]; then
        echo "ERROR: demo-server start-multi did not write sidecar within ${POLL_TIMEOUT}s." >&2
        echo "       Check ${DEMO_RUN_DIR}/dtu-server.log for details." >&2
        echo "       Common cause: port conflict — stop other services on the demo ports." >&2
        exit 1
    fi
done

echo "    Nested sidecar ready after ${POLL_ELAPSED}s"

# ---------------------------------------------------------------------------
# Print DTU clone URLs (nested {org_slug: {sensor_id: url}})
# ---------------------------------------------------------------------------

echo ""
echo "==> DTU clone URLs:"
# Parse nested JSON with python3 (available on all macOS/Linux demo targets).
if command -v python3 &>/dev/null; then
    python3 - "${URLS_MULTI_FILE}" << 'PYEOF'
import json, sys
with open(sys.argv[1]) as f:
    nested = json.load(f)
for org_slug in sorted(nested.keys()):
    for sensor_id, url in sorted(nested[org_slug].items()):
        print(f"    {org_slug}/{sensor_id:15s}: {url}")
PYEOF
else
    cat "${URLS_MULTI_FILE}"
fi
echo ""

# ---------------------------------------------------------------------------
# Generate per-org overlay TOMLs (N×M, BC-2.06.012 / BC-2.06.013 / BC-2.06.014)
#
# DTU ports are ephemeral — only known now (after urls-multi.json is ready).
# prism's spec loader reads customers_dir = spec_dir + "/customers" at boot step 4c
# (crates/prism-bin/src/boot.rs). Each sensor overlay sets base_url to the DTU port.
# Overlay format: BC-2.06.012 / ADR-029 scalar-only hybrid instance overlay.
#
# GAP-3: DEMO_RUN_DIR is threaded into the Python block via os.environ so the Python
# process can open the sidecar from the correct location without hardcoding the path.
# The heredoc uses single-quotes ('PYEOF') so shell does NOT expand ${...} inside —
# os.environ["DEMO_RUN_DIR"] and os.environ["DEMO_CONFIG_DIR"] are read at runtime.
# ---------------------------------------------------------------------------

DEMO_SPECS_DIR="${DEMO_CONFIG_DIR}/specs"
DEMO_CUSTOMERS_DIR="${DEMO_SPECS_DIR}/customers"

echo "==> Generating per-org sensor overlays (base_url → DTU ports)..."

# Export vars for the Python block (GAP-3: nested heredoc reads from env, not from
# shell variable interpolation, so DEMO_RUN_DIR is correctly threaded in).
export DEMO_RUN_DIR
export DEMO_CONFIG_DIR

python3 - << 'PYEOF'
import json, os, sys

urls_multi_file = os.path.join(os.environ["DEMO_RUN_DIR"], ".prism-dtu-demo-server.urls-multi.json")
customers_dir   = os.path.join(os.environ["DEMO_CONFIG_DIR"], "specs", "customers")

with open(urls_multi_file) as f:
    nested = json.load(f)  # {"org-a": {"crowdstrike": "http://...", ...}, ...}

for org_slug, sensor_map in nested.items():
    org_dir = os.path.join(customers_dir, org_slug)
    os.makedirs(org_dir, exist_ok=True)
    for sensor_id, base_url in sensor_map.items():
        overlay_path = os.path.join(org_dir, f"{sensor_id}.sensor.toml")
        # BC-2.06.013: scalar-only overlay — no auth_type, no table sections.
        # BC-2.06.012: prism reads this at boot step 4c via customers_dir.
        # BC-2.06.014: each (org_slug, sensor_id) has a distinct base_url.
        content = (
            f"# Per-org overlay for {sensor_id} sensor — {org_slug} (DTU demo)\n"
            f"# BC-2.06.013: scalar-only overlay; no schema fields or auth_type.\n"
            f"# Generated by demo-run.sh after DTU server starts (ephemeral port).\n"
            f'extends     = "{sensor_id}"\n'
            f'instance_id = "{sensor_id}@{org_slug}"\n'
            f'base_url    = "{base_url}"\n'
        )
        with open(overlay_path, "w") as out:
            out.write(content)
        print(f"    Overlay written: {overlay_path}")

PYEOF

echo "    Sensor overlays ready at ${DEMO_CUSTOMERS_DIR}/"
echo ""

# ---------------------------------------------------------------------------
# Print prism-bin start command
#
# WHY env vars are required here (F-HIGH-301):
#   The 4 TYPE specs (crowdstrike/armis/claroty/cyberint) use ${env.VAR} tokens
#   in their base_url fields. env_resolver.rs resolves these at boot STEP 4a —
#   before step 4c overlay loading. If any var is absent or empty, E-SPEC-024
#   fires and boot hard-aborts (exit 2) before the per-org overlay can replace
#   base_url with the actual DTU clone URL.
#
#   CROWDSTRIKE_BASE_URL must be "http://127.0.0.1": the crowdstrike-oauth2 plugin
#   manifest allowed_urls = ["api.crowdstrike.com", "127.0.0.1"] — SEC-003 validates
#   the TYPE spec base_url host against this list at step 7.5b.
#
#   ARMIS_INSTANCE_URL and CLAROTY_INSTANCE_URL are resolved by env_resolver at
#   step 4a (E-SPEC-024 guard). Per-org overlays replace base_url at step 4c.
#
#   CYBERINT_ENVIRONMENT is interpolated into the base_url template:
#   "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io" at step 4a.
#
#   Values mirrored from the proven E2E test harness:
#   crates/prism-bin/tests/helpers/mod.rs (the authoritative env-var source-of-truth).
# ---------------------------------------------------------------------------

PRISM_BIN="${REPO_ROOT}/target/release/prism"

echo "==> To start prism, run in a new terminal:"
echo ""
echo "    CROWDSTRIKE_BASE_URL=http://127.0.0.1 \\"
echo "    ARMIS_INSTANCE_URL=http://127.0.0.1 \\"
echo "    CLAROTY_INSTANCE_URL=http://127.0.0.1 \\"
echo "    CYBERINT_ENVIRONMENT=demo \\"
echo "    ${PRISM_BIN} --config-dir ${DEMO_CONFIG_DIR} start"
echo ""
echo "    Then add prism to Claude Code (see docs/DEMO-RUNBOOK.md §4)."
echo ""
echo "==> DTU server log: ${DEMO_RUN_DIR}/dtu-server.log"
echo "    To stop everything: bash scripts/demo-teardown.sh"
echo ""
