#!/usr/bin/env bash
# scripts/demo-setup.sh — One-time idempotent setup for the Prism DTU demo environment.
#
# USAGE
#   bash scripts/demo-setup.sh [--config-dir DIR]
#
# WHAT IT DOES (in order)
#   1. Verify prerequisites (cargo)
#   2. Build prism and prism-dtu-demo-server (release profile)
#   3. Create demo config directory structure
#   4. Copy sensor TOML specs to <DIR>/specs/
#   5. Copy crowdstrike-oauth2.prx plugin + write crowdstrike-oauth2.manifest.toml
#   6. Provision infusion TOMLs (threatintel + nvd) to <DIR>/infusions/ and
#      copy threatintel-lookup.prx + manifest to <DIR>/plugins/ (fixes boot
#      "infusion count: 0" — InfusionLoader reads {config_dir}/infusions/)
#   7. Write <DIR>/prism.toml (3-org: org-a, org-b, org-c)
#   8. Bootstrap N×M dummy credentials in OS keyring via `prism credential set`
#   9. Print next-step instructions
#
# IDEMPOTENCY
#   Safe to run multiple times. mkdir -p is used for all directories.
#   Keyring writes overwrite existing entries.
#   TOML and spec files are overwritten.
#
# CREDENTIAL TABLE (N=3 orgs, M=variable sensors per org — 10 total):
#   org-a: crowdstrike (client_id, client_secret), armis (bearer_token)        — 3
#   org-b: claroty (bearer_token), cyberint (api_key)                           — 2
#   org-c: crowdstrike (client_id, client_secret), armis (bearer_token),
#          claroty (bearer_token), cyberint (api_key)                           — 5
#   Total: 10
#
# Cyberint api_key values MUST match initial_access_token in scripts/demo.toml:
#   org-b: "demo-cyberint-api-key-org-b"   (matches [orgs.org-b].initial_access_token)
#   org-c: "demo-cyberint-api-key-org-c"   (matches [orgs.org-c].initial_access_token)
#
# EC-003: If crowdstrike-oauth2.prx is not found, exits 1 with an actionable message.
# AC-008: passes shellcheck with zero errors/warnings.
# AD-017: all credential values are piped via stdin, never passed as CLI argv.
#
# Stories: S-DEMO-003 | S-DEMO-LAUNCHER-CONSOLIDATION-001 | BCs: BC-2.06.001, BC-2.03.007

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# DEFECT-DEMOSETUP-CWD-001: cd to repo root immediately so that `cargo build`
# invocations below succeed regardless of the caller's working directory.
# All subsequent paths are absolute-var-based (${REPO_ROOT}/..., ${DEMO_CONFIG_DIR}/...)
# so this cd is safe and does not affect any other operation.
cd "$REPO_ROOT"

# Demo config directory (override with --config-dir)
DEMO_CONFIG_DIR="${HOME}/.config/prism-demo"

# Parse optional --config-dir argument
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

DEMO_SPECS_DIR="${DEMO_CONFIG_DIR}/specs"
DEMO_STATE_DIR="${DEMO_CONFIG_DIR}/state"
DEMO_PLUGINS_DIR="${DEMO_CONFIG_DIR}/plugins"

# 3-org UUIDs — must be valid UUID v7 (time-ordered) to pass boot step 3.
# Must match [orgs.*].org_id in scripts/demo.toml.
ORG_A_SLUG="org-a"
ORG_A_ID="0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0000"
ORG_B_SLUG="org-b"
ORG_B_ID="0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0001"
ORG_C_SLUG="org-c"
ORG_C_ID="0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0002"

# Plugin artifact path (committed by S-PLUGIN-CI-001)
PLUGIN_ARTIFACT="${REPO_ROOT}/crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx"

# Prism binary
PRISM_BIN="${REPO_ROOT}/target/release/prism"

# ---------------------------------------------------------------------------
# Step 1: Prerequisites check
# ---------------------------------------------------------------------------

echo "==> [1/9] Checking prerequisites..."

if ! command -v cargo &>/dev/null; then
    echo "ERROR: cargo not found. Install Rust from https://rustup.rs" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Step 2: Build prism (release)
# ---------------------------------------------------------------------------

echo "==> [2/9] Building prism (release)..."
cargo build --release -p prism-bin 2>&1

echo "    Building prism-dtu-demo-server (release, features: dtu,fixture-gen)..."
cargo build --release -p prism-dtu-demo-server --features dtu,fixture-gen 2>&1

# ---------------------------------------------------------------------------
# Step 3: Create config directory structure
# ---------------------------------------------------------------------------

echo "==> [3/9] Creating demo config directory at ${DEMO_CONFIG_DIR}..."
mkdir -p "${DEMO_CONFIG_DIR}"
mkdir -p "${DEMO_SPECS_DIR}"
mkdir -p "${DEMO_STATE_DIR}"
mkdir -p "${DEMO_PLUGINS_DIR}"
mkdir -p "${DEMO_CONFIG_DIR}/run"
# Create org-slug subdirectories under customers/ (overlays written later by demo-run.sh)
mkdir -p "${DEMO_SPECS_DIR}/customers/${ORG_A_SLUG}"
mkdir -p "${DEMO_SPECS_DIR}/customers/${ORG_B_SLUG}"
mkdir -p "${DEMO_SPECS_DIR}/customers/${ORG_C_SLUG}"

# ---------------------------------------------------------------------------
# Step 4: Copy sensor TOML specs
# ---------------------------------------------------------------------------

echo "==> [4/9] Copying sensor TOML specs to ${DEMO_SPECS_DIR}/..."
cp "${REPO_ROOT}/crates/prism-sensors/specs/crowdstrike.sensor.toml" "${DEMO_SPECS_DIR}/"
cp "${REPO_ROOT}/crates/prism-sensors/specs/armis.sensor.toml" "${DEMO_SPECS_DIR}/"
cp "${REPO_ROOT}/crates/prism-sensors/specs/claroty.sensor.toml" "${DEMO_SPECS_DIR}/"
cp "${REPO_ROOT}/crates/prism-sensors/specs/cyberint.sensor.toml" "${DEMO_SPECS_DIR}/"

echo "    Sensor specs copied: crowdstrike, armis, claroty, cyberint"

# ---------------------------------------------------------------------------
# Step 5: Copy crowdstrike-oauth2.prx plugin
# ---------------------------------------------------------------------------

echo "==> [5/9] Copying crowdstrike-oauth2.prx plugin to ${DEMO_PLUGINS_DIR}/..."

# EC-003: Exit 1 with actionable message if plugin artifact not found.
if [[ ! -f "${PLUGIN_ARTIFACT}" ]]; then
    echo "ERROR: Plugin artifact not found at ${PLUGIN_ARTIFACT}" >&2
    echo "       Run: cargo build -p prism-spec-engine --features wasm-plugins" >&2
    echo "       Then re-run this script." >&2
    exit 1
fi

cp "${PLUGIN_ARTIFACT}" "${DEMO_PLUGINS_DIR}/"
echo "    crowdstrike-oauth2.prx copied"

# Write a DTU-safe companion manifest that extends the production allowed_urls with
# "127.0.0.1" so the plugin SEC-003 check passes when the CrowdStrike DTU clone is
# the OAuth2 token endpoint (demo.toml: bind = "127.0.0.1").
# The production plugin.toml has allowed_urls = ["api.crowdstrike.com"] and is NOT
# modified. Only this staging copy in DEMO_PLUGINS_DIR has the extended allowlist.
# Naming convention: {prx_stem}.manifest.toml (load_all_plugins convention in
# crates/prism-spec-engine/src/plugin/mod.rs path.with_extension("manifest.toml")).
cat > "${DEMO_PLUGINS_DIR}/crowdstrike-oauth2.manifest.toml" << 'MANIFESTEOF'
# DTU-safe companion manifest for crowdstrike-oauth2 demo plugin.
# Extends production allowed_urls with "127.0.0.1" so SEC-003 passes when the
# CrowdStrike DTU clone is the token endpoint (demo.toml: bind = "127.0.0.1").
# The production plugin.toml is NOT modified — only this demo-staging copy differs.
# Mirrors the E2E test pattern in crates/prism-bin/tests/helpers/mod.rs:stage_crowdstrike_plugin.
name = "crowdstrike-oauth2"
version = "0.1.0"
format_version = 1
plugin_type = "sensor_auth"
allowed_urls = ["api.crowdstrike.com", "127.0.0.1"]
MANIFESTEOF

echo "    crowdstrike-oauth2.manifest.toml written (DTU-safe SEC-003 allowlist)"

# ---------------------------------------------------------------------------
# Step 6: Provision infusion TOMLs and threatintel plugin
#
# InfusionLoader loads from {config_dir}/infusions/ (boot.rs step 7.6).
# Without this step, boot shows "infusion count: 0" and enrichment queries fail.
#
# Idempotent: mkdir -p + cp overwrites on re-run.
# ---------------------------------------------------------------------------

DEMO_INFUSIONS_DIR="${DEMO_CONFIG_DIR}/infusions"

echo "==> [6/9] Provisioning infusion specs and threatintel plugin to ${DEMO_INFUSIONS_DIR}/..."
mkdir -p "${DEMO_INFUSIONS_DIR}"

# Copy the enrichment infusion TOML specs.
# InfusionLoader::load_all expects *.infusion.toml files under {config_dir}/infusions/.
cp "${REPO_ROOT}/specs/infusions/threatintel.infusion.toml" "${DEMO_INFUSIONS_DIR}/"
cp "${REPO_ROOT}/specs/infusions/nvd.infusion.toml" "${DEMO_INFUSIONS_DIR}/"
echo "    Infusion specs copied: threatintel.infusion.toml, nvd.infusion.toml"

# Copy the threatintel WASM plugin .prx + its companion manifest.
# PluginRuntime::load_all_plugins reads pairs: *.prx + *.manifest.toml.
# The plugin must be present before prism boot loads the infusion registry (boot step 7.6).
#
# Artifact source: built by `just build-plugin-threatintel-infusion`
#   → crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx
#   → crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml
THREATINTEL_PRX="${REPO_ROOT}/crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.prx"
THREATINTEL_MANIFEST="${REPO_ROOT}/crates/prism-spec-engine/plugins/threatintel-lookup/threatintel-lookup.manifest.toml"

if [[ ! -f "${THREATINTEL_PRX}" ]]; then
    echo "ERROR: threatintel-lookup.prx not found at ${THREATINTEL_PRX}" >&2
    echo "       Run: just build-plugin-threatintel-infusion" >&2
    echo "       Then re-run this script." >&2
    exit 1
fi

cp "${THREATINTEL_PRX}" "${DEMO_PLUGINS_DIR}/threatintel-lookup.prx"
cp "${THREATINTEL_MANIFEST}" "${DEMO_PLUGINS_DIR}/threatintel-lookup.manifest.toml"
echo "    threatintel-lookup.prx + manifest copied to ${DEMO_PLUGINS_DIR}/"

# ---------------------------------------------------------------------------
# Step 7: Write multi-org prism.toml (3 orgs: org-a, org-b, org-c)
#
# BC-2.06.001: generated prism.toml must be schema-valid (N [[orgs]] entries).
# ---------------------------------------------------------------------------

echo "==> [7/9] Writing ${DEMO_CONFIG_DIR}/prism.toml (3-org: org-a, org-b, org-c)..."

cat > "${DEMO_CONFIG_DIR}/prism.toml" << TOMLEOF
spec_dir   = "${DEMO_SPECS_DIR}"
state_dir  = "${DEMO_STATE_DIR}"
plugin_dir = "${DEMO_PLUGINS_DIR}"

[[orgs]]
org_id   = "${ORG_A_ID}"
org_slug = "${ORG_A_SLUG}"

[[orgs]]
org_id   = "${ORG_B_ID}"
org_slug = "${ORG_B_SLUG}"

[[orgs]]
org_id   = "${ORG_C_ID}"
org_slug = "${ORG_C_SLUG}"
TOMLEOF

echo "    prism.toml written (3 [[orgs]] entries)"

# ---------------------------------------------------------------------------
# Step 8: Bootstrap N×M credentials (dummy values for DTU demo)
#
# 10 total credentials — one per (org_slug, sensor, credential_name) combination.
# AD-017: values piped via stdin (rpassword reads from piped stdin in non-TTY mode).
# Cyberint api_key values MUST match initial_access_token in scripts/demo.toml.
# ---------------------------------------------------------------------------

echo "==> [8/9] Bootstrapping demo credentials in OS keyring (10 total)..."
echo "    (Values are dummy credentials safe for DTU use only)"
echo "    NOTE: If the keyring is unavailable, set env vars instead."
echo "          See docs/DEMO-RUNBOOK.md §Troubleshooting for details."

# Helper to set a credential — reads from stdin (AD-017 compliant).
# Usage: set_cred <org_slug> <sensor> <name> <value>
set_cred() {
    local org_slug="$1"
    local sensor="$2"
    local name="$3"
    local value="$4"
    # AD-017: the value is piped via stdin, never passed as a CLI arg.
    # rpassword reads from piped stdin in non-TTY mode.
    if printf '%s\n' "${value}" | "${PRISM_BIN}" \
        --config-dir "${DEMO_CONFIG_DIR}" \
        credential set \
        --sensor "${sensor}" \
        --name "${name}" \
        --org-slug "${org_slug}" \
        2>/dev/null; then
        echo "    Stored: prism/${org_slug}/${sensor}/${name}"
    else
        # BC-2.06.003 Tier 2 canonical env var format:
        # PRISM_CLIENTS_{ORG_UPPER}_SENSORS_{SENSOR_UPPER}_{NAME_UPPER}
        # where org slug uppercased with hyphens → underscores (ADR-032).
        local org_upper
        org_upper="$(printf '%s' "${org_slug}" | tr '[:lower:]-' '[:upper:]_')"
        local sensor_upper
        sensor_upper="$(printf '%s' "${sensor}" | tr '[:lower:]-' '[:upper:]_')"
        local name_upper
        name_upper="$(printf '%s' "${name}" | tr '[:lower:]-' '[:upper:]_')"
        echo "    WARN: keyring write failed for prism/${org_slug}/${sensor}/${name}" \
             "(use env var PRISM_CLIENTS_${org_upper}_SENSORS_${sensor_upper}_${name_upper} as fallback)" >&2
    fi
}

# org-a: crowdstrike (client_id, client_secret), armis (bearer_token) — 3 credentials
set_cred "${ORG_A_SLUG}" "crowdstrike" "client_id"     "demo-cs-client-id-org-a"
set_cred "${ORG_A_SLUG}" "crowdstrike" "client_secret" "demo-cs-client-secret-org-a"
set_cred "${ORG_A_SLUG}" "armis"       "bearer_token"  "demo-armis-bearer-token-org-a"

# org-b: claroty (bearer_token), cyberint (api_key) — 2 credentials
# Cyberint api_key MUST match [orgs.org-b].initial_access_token in scripts/demo.toml.
set_cred "${ORG_B_SLUG}" "claroty"   "bearer_token" "demo-claroty-bearer-token-org-b"
set_cred "${ORG_B_SLUG}" "cyberint"  "api_key"      "demo-cyberint-api-key-org-b"

# org-c: crowdstrike (client_id, client_secret), armis (bearer_token),
#        claroty (bearer_token), cyberint (api_key) — 5 credentials
# Cyberint api_key MUST match [orgs.org-c].initial_access_token in scripts/demo.toml.
set_cred "${ORG_C_SLUG}" "crowdstrike" "client_id"     "demo-cs-client-id-org-c"
set_cred "${ORG_C_SLUG}" "crowdstrike" "client_secret" "demo-cs-client-secret-org-c"
set_cred "${ORG_C_SLUG}" "armis"       "bearer_token"  "demo-armis-bearer-token-org-c"
set_cred "${ORG_C_SLUG}" "claroty"     "bearer_token"  "demo-claroty-bearer-token-org-c"
set_cred "${ORG_C_SLUG}" "cyberint"    "api_key"       "demo-cyberint-api-key-org-c"

echo "    Credentials bootstrapped (10 entries)"

# ---------------------------------------------------------------------------
# Step 9: Print instructions
# ---------------------------------------------------------------------------

echo ""
echo "==> [9/9] Setup complete!"
echo ""
echo "    Config dir provisioned: ${DEMO_CONFIG_DIR}"
echo "    Run the following commands from the repo root (${REPO_ROOT})"
echo ""
echo "Next steps:"
echo ""
echo "  1. Start the DTU demo server and generate per-org overlays:"
echo "       bash scripts/demo-run.sh --config-dir \"${DEMO_CONFIG_DIR}\""
echo ""
echo "  2. Add prism to Claude Code (see docs/DEMO-RUNBOOK.md §Connecting Claude Code)"
echo ""
echo "  3. In Claude Code, invoke the query MCP tool (see docs/DEMO-RUNBOOK.md §5):"
echo "       query  \"SELECT * FROM crowdstrike_detections LIMIT 5\""
echo ""
echo "  4. To tear down:"
echo "       bash scripts/demo-teardown.sh --config-dir \"${DEMO_CONFIG_DIR}\""
echo ""
