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
    # "_global" holds enrichment DTU URLs (ENRICH-3); skip it in overlay generation.
    # Global enrichment DTUs are NOT per-org sensors — writing overlays for them would
    # create bogus {customers_dir}/_global/{enrichment_name}.sensor.toml files that
    # prism would try to parse as sensor specs (which they are not). demo-run.sh reads
    # _global entries separately to export PRISM_THREATINTEL_BASE_URL etc.
    if org_slug == "_global":
        continue
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
# Extract global enrichment DTU URLs from the _global sidecar key (ENRICH-3)
#
# start-multi emits enrichment DTU URLs under "_global" in the nested sidecar
# (not under any org slug — per write_multi_url_sidecar_to_path). We extract them
# and:
#   1. Export PRISM_THREATINTEL_BASE_URL for the ThreatIntel plugin (PluginConfigMap,
#      [[infusion.credentials]] field_name="base_url", ENRICH-2).
#   2. Export PRISM_THREATINTEL_API_KEY (fixed demo value; real value via demo-setup.sh).
#   3. Write an override nvd.infusion.toml with the DTU base_url into
#      {config_dir}/infusions/ — because the http_lookup loader does NOT resolve
#      ${env.*} tokens in base_url (env_resolver.rs only processes SensorSpec fields,
#      not InfusionSpec). The override replaces base_url at prism boot time.
# ---------------------------------------------------------------------------

echo "==> Extracting global enrichment DTU URLs..."

# Export SCRIPT_DIR and DEMO_CONFIG_DIR for the Python block (PYEOF uses single-quotes
# so shell does NOT expand ${...} inside — os.environ reads at runtime).
export SCRIPT_DIR
export DEMO_CONFIG_DIR

PRISM_THREATINTEL_BASE_URL=""
PRISM_NVD_BASE_URL=""

if command -v python3 &>/dev/null; then
    _enrichment_vars="$(python3 - "${URLS_MULTI_FILE}" << 'PYEOF'
import json, os, sys

urls_multi_file = sys.argv[1]
config_dir = os.environ["DEMO_CONFIG_DIR"]
script_dir = os.environ["SCRIPT_DIR"]

with open(urls_multi_file) as f:
    nested = json.load(f)

global_urls = nested.get("_global", {})
threatintel_url = global_urls.get("threatintel", "")
nvd_url = global_urls.get("nvd", "")

# Print shell variable assignments for eval (each line safe for eval).
print(f"PRISM_THREATINTEL_BASE_URL={threatintel_url!r}")
print(f"PRISM_NVD_BASE_URL={nvd_url!r}")

# Write override nvd.infusion.toml with the DTU base_url.
# The http_lookup loader does not resolve ${env.*} tokens in base_url, so we write
# a concrete TOML override with the actual DTU endpoint into the infusions directory.
if nvd_url:
    infusions_dir = os.path.join(config_dir, "infusions")
    os.makedirs(infusions_dir, exist_ok=True)
    override_path = os.path.join(infusions_dir, "nvd.infusion.toml")
    # Canonical spec is at specs/infusions/nvd.infusion.toml relative to repo root.
    canonical_path = os.path.join(script_dir, "..", "specs", "infusions", "nvd.infusion.toml")
    try:
        with open(canonical_path) as f:
            canonical_toml = f.read()
        # Replace the production base_url line with the DTU base_url.
        # This is a 1:1 substitution — the rest of the spec is unchanged.
        override_toml = canonical_toml.replace(
            'base_url      = "https://services.nvd.nist.gov"',
            f'base_url      = "{nvd_url}"',
        )
        with open(override_path, "w") as out:
            out.write("# AUTO-GENERATED by demo-run.sh — DO NOT EDIT\n")
            out.write("# Overrides base_url to DTU clone endpoint (ENRICH-2/ENRICH-3).\n")
            out.write(f"# Source: {canonical_path}\n\n")
            out.write(override_toml)
        print(f"# NVD override TOML written: {override_path}")
    except Exception as e:
        print(f"# WARNING: could not write NVD override TOML: {e}", file=sys.stderr)
PYEOF
)"
    # Evaluate only the assignment lines (lines starting with PRISM_); ignore comment lines.
    while IFS= read -r line; do
        case "${line}" in
            PRISM_*)
                eval "${line}"
                ;;
        esac
    done <<< "${_enrichment_vars}"
    # Echo the informational lines (comment lines starting with #).
    while IFS= read -r line; do
        case "${line}" in
            '#'*)
                echo "    ${line#'# '}"
                ;;
        esac
    done <<< "${_enrichment_vars}"
fi

if [[ -n "${PRISM_THREATINTEL_BASE_URL}" ]]; then
    echo "    ThreatIntel DTU URL:  ${PRISM_THREATINTEL_BASE_URL}"
else
    echo "    WARNING: no ThreatIntel global DTU URL in sidecar (_global.threatintel missing)" >&2
fi
if [[ -n "${PRISM_NVD_BASE_URL}" ]]; then
    echo "    NVD DTU URL:          ${PRISM_NVD_BASE_URL}"
else
    echo "    WARNING: no NVD global DTU URL in sidecar (_global.nvd missing)" >&2
fi
echo ""

# PRISM_THREATINTEL_API_KEY: fixed demo value.
# AD-017: in real deployments, the value is piped via stdin or read from keyring;
# the demo uses a well-known test key that matches the DTU server's allow-list.
# The plugin WASM reads this from PluginConfigMap["api_key"] (ENRICH-2,
# [[infusion.credentials]] field_name="api_key", env_var="PRISM_THREATINTEL_API_KEY").
PRISM_THREATINTEL_API_KEY="demo-threatintel-api-key"

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
echo "    PRISM_THREATINTEL_BASE_URL=${PRISM_THREATINTEL_BASE_URL} \\"
echo "    PRISM_THREATINTEL_API_KEY=${PRISM_THREATINTEL_API_KEY} \\"
echo "    ${PRISM_BIN} --config-dir ${DEMO_CONFIG_DIR} start"
echo ""
echo "    Then add prism to Claude Code (see docs/DEMO-RUNBOOK.md §4)."
echo ""
echo "==> DTU server log: ${DEMO_RUN_DIR}/dtu-server.log"
echo "    To stop everything: bash scripts/demo-teardown.sh"
echo ""
