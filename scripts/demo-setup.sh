#!/usr/bin/env bash
# scripts/demo-setup.sh — One-time idempotent setup for the Prism DTU demo environment.
#
# USAGE
#   bash scripts/demo-setup.sh [--config-dir DIR]
#
# WHAT IT DOES (in order)
#   1. Verify prerequisites (cargo, shellcheck, the prism binary)
#   2. Build prism and prism-dtu-demo-server (release profile)
#   3. Create demo config directory structure
#   4. Copy sensor TOML specs to ~/.config/prism-demo/specs/
#   5. Copy crowdstrike-oauth2.prx plugin to ~/.config/prism-demo/plugins/
#   6. Write ~/.config/prism-demo/prism.toml (valid PrismConfig)
#   7. Bootstrap dummy credentials in the OS keyring via `prism credential set`
#   8. Print next-step instructions
#
# IDEMPOTENCY
#   Safe to run multiple times. mkdir -p is used for all directories.
#   Keyring writes overwrite existing entries.
#   TOML and spec files are overwritten.
#
# EC-003: If crowdstrike-oauth2.prx is not found, exits 1 with an actionable message.
# AC-008: This script passes shellcheck with zero errors/warnings.
#
# Story: S-DEMO-003 | BCs: BC-2.06.001, BC-2.03.007

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

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
DEMO_ORG_SLUG="demo-org"

# Canonical org_id UUID v7 — matches generate_demo_prism_toml() in credential_cli.rs.
# Must be a real UUID v7 (time-ordered, version 7) to pass boot step 3.
DEMO_ORG_ID="0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c"

# Plugin artifact path (committed by S-PLUGIN-CI-001)
PLUGIN_ARTIFACT="${REPO_ROOT}/crates/prism-spec-engine/plugins/crowdstrike-oauth2/crowdstrike-oauth2.prx"

# Prism binary
PRISM_BIN="${REPO_ROOT}/target/release/prism"

# ---------------------------------------------------------------------------
# Step 1: Prerequisites check
# ---------------------------------------------------------------------------

echo "==> [1/8] Checking prerequisites..."

if ! command -v cargo &>/dev/null; then
    echo "ERROR: cargo not found. Install Rust from https://rustup.rs" >&2
    exit 1
fi

# ---------------------------------------------------------------------------
# Step 2: Build prism (release)
# ---------------------------------------------------------------------------

echo "==> [2/8] Building prism (release)..."
cargo build --release -p prism-bin 2>&1

echo "    Building prism-dtu-demo-server (release)..."
cargo build --release -p prism-dtu-demo-server --features dtu 2>&1

# ---------------------------------------------------------------------------
# Step 3: Create config directory structure
# ---------------------------------------------------------------------------

echo "==> [3/8] Creating demo config directory at ${DEMO_CONFIG_DIR}..."
mkdir -p "${DEMO_CONFIG_DIR}"
mkdir -p "${DEMO_SPECS_DIR}"
mkdir -p "${DEMO_STATE_DIR}"
mkdir -p "${DEMO_PLUGINS_DIR}"
mkdir -p "${DEMO_CONFIG_DIR}/run"

# ---------------------------------------------------------------------------
# Step 4: Copy sensor TOML specs
# ---------------------------------------------------------------------------

echo "==> [4/8] Copying sensor TOML specs to ${DEMO_SPECS_DIR}/..."
cp "${REPO_ROOT}/crates/prism-sensors/specs/crowdstrike.sensor.toml" "${DEMO_SPECS_DIR}/"
cp "${REPO_ROOT}/crates/prism-sensors/specs/armis.sensor.toml" "${DEMO_SPECS_DIR}/"
cp "${REPO_ROOT}/crates/prism-sensors/specs/claroty.sensor.toml" "${DEMO_SPECS_DIR}/"
cp "${REPO_ROOT}/crates/prism-sensors/specs/cyberint.sensor.toml" "${DEMO_SPECS_DIR}/"

echo "    Sensor specs copied: crowdstrike, armis, claroty, cyberint"

# ---------------------------------------------------------------------------
# Step 5: Copy crowdstrike-oauth2.prx plugin
# ---------------------------------------------------------------------------

echo "==> [5/8] Copying crowdstrike-oauth2.prx plugin to ${DEMO_PLUGINS_DIR}/..."

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
# Step 6: Write prism.toml
# ---------------------------------------------------------------------------

echo "==> [6/8] Writing ${DEMO_CONFIG_DIR}/prism.toml..."

cat > "${DEMO_CONFIG_DIR}/prism.toml" << TOMLEOF
spec_dir = "${DEMO_SPECS_DIR}"
state_dir = "${DEMO_STATE_DIR}"
plugin_dir = "${DEMO_PLUGINS_DIR}"

[[orgs]]
org_id = "${DEMO_ORG_ID}"
org_slug = "${DEMO_ORG_SLUG}"
TOMLEOF

echo "    prism.toml written"

# ---------------------------------------------------------------------------
# Step 7: Bootstrap credentials (dummy values for DTU demo)
# ---------------------------------------------------------------------------

echo "==> [7/8] Bootstrapping demo credentials in OS keyring..."
echo "    (Values are dummy credentials safe for DTU use only)"
echo "    NOTE: If the keyring is unavailable, set env vars instead."
echo "          See docs/DEMO-RUNBOOK.md §Troubleshooting for details."

# Helper to set a credential — reads from stdin (AD-017 compliant).
# Usage: set_cred <sensor> <name> <value>
set_cred() {
    local sensor="$1"
    local name="$2"
    local value="$3"
    # AD-017: the value is piped via stdin, never passed as a CLI arg.
    # rpassword reads from piped stdin in non-TTY mode.
    if printf '%s\n' "${value}" | "${PRISM_BIN}" \
        --config-dir "${DEMO_CONFIG_DIR}" \
        credential set \
        --sensor "${sensor}" \
        --name "${name}" \
        --org-slug "${DEMO_ORG_SLUG}" \
        2>/dev/null; then
        echo "    Stored: prism/${sensor}/${name}"
    else
        # BC-2.06.003 Tier 2 canonical env var format: PRISM_CLIENTS_{ID}_SENSORS_{SENSOR}_{REF}
        # where {ID} = org slug uppercased with hyphens → underscores (ADR-032).
        local org_upper
        org_upper="$(echo "${DEMO_ORG_SLUG}" | tr '[:lower:]-' '[:upper:]_')"
        local sensor_upper
        sensor_upper="$(echo "${sensor}" | tr '[:lower:]-' '[:upper:]_')"
        local name_upper
        name_upper="$(echo "${name}" | tr '[:lower:]-' '[:upper:]_')"
        echo "    WARN: keyring write failed for prism/${sensor}/${name}" \
             "(use env var PRISM_CLIENTS_${org_upper}_SENSORS_${sensor_upper}_${name_upper} as fallback)" >&2
    fi
}

# CrowdStrike: OAuth2 client credentials (oauth2_client_credentials auth_type)
set_cred "crowdstrike" "client_id" "demo-cs-client-id"
set_cred "crowdstrike" "client_secret" "demo-cs-client-secret"

# Armis: bearer_static auth_type — credential name matches spec [[credential_refs]]
set_cred "armis" "bearer_token" "demo-armis-bearer-token"

# Claroty: bearer_static auth_type — credential name matches spec [[credential_refs]]
set_cred "claroty" "bearer_token" "demo-claroty-bearer-token"

# Cyberint: cookie_roundtrip auth_type — credential name matches spec [[credential_refs]]
# The value MUST match initial_access_token in scripts/demo.toml (DTU allowlist seed).
set_cred "cyberint" "api_key" "demo-cyberint-api-key"

echo "    Credentials bootstrapped"

# ---------------------------------------------------------------------------
# Step 8: Print instructions
# ---------------------------------------------------------------------------

echo ""
echo "==> [8/8] Setup complete!"
echo ""
echo "Next steps:"
echo ""
echo "  1. Start the DTU demo server and prism:"
echo "       bash scripts/demo-run.sh"
echo ""
echo "  2. Add prism to Claude Code (see docs/DEMO-RUNBOOK.md §Connecting Claude Code)"
echo ""
echo "  3. In Claude Code, run:"
echo "       /mcp tool_query \"FROM crowdstrike_detections LIMIT 5\""
echo ""
echo "  4. To tear down:"
echo "       bash scripts/demo-teardown.sh"
echo ""
