#!/usr/bin/env bash
# scripts/demo-teardown.sh — Remove the Prism DTU demo environment.
#
# USAGE
#   bash scripts/demo-teardown.sh [--config-dir DIR]
#
# WHAT IT DOES
#   1. Kills the DTU demo server (via PID file)
#   2. Removes ~/.config/prism-demo/ (config, specs, plugins, state)
#   3. Deletes the 5 demo OS keyring entries via platform keyring CLI
#   4. Exits 0
#
# AC-007: DTU server is killed; config dir removed; keyring entries deleted.
# AC-008: passes shellcheck with zero errors/warnings.
#
# Story: S-DEMO-003 | BC: BC-2.03.005

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

DEMO_CONFIG_DIR="${HOME}/.config/prism-demo"
DEMO_ORG_SLUG="demo-org"

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
PID_FILE="${DEMO_RUN_DIR}/.prism-dtu-demo-server.pid"
PRISM_BIN="${REPO_ROOT}/target/release/prism"

# ---------------------------------------------------------------------------
# Step 1: Kill DTU demo server
# ---------------------------------------------------------------------------

echo "==> [1/3] Stopping DTU demo server..."

if [[ -f "${PID_FILE}" ]]; then
    DTU_PID=$(cat "${PID_FILE}")
    if kill -0 "${DTU_PID}" 2>/dev/null; then
        kill "${DTU_PID}" 2>/dev/null || true
        # Wait up to 5s for clean shutdown.
        WAIT_COUNT=0
        while kill -0 "${DTU_PID}" 2>/dev/null && [[ "${WAIT_COUNT}" -lt 5 ]]; do
            sleep 1
            WAIT_COUNT=$((WAIT_COUNT + 1))
        done
        if kill -0 "${DTU_PID}" 2>/dev/null; then
            kill -9 "${DTU_PID}" 2>/dev/null || true
            echo "    DTU server force-killed (PID ${DTU_PID})"
        else
            echo "    DTU server stopped (PID ${DTU_PID})"
        fi
    else
        echo "    DTU server not running (PID ${DTU_PID} not found)"
    fi
    rm -f "${PID_FILE}"
else
    echo "    No DTU PID file found — server may not be running"
fi

# ---------------------------------------------------------------------------
# Step 2: Remove config directory
# ---------------------------------------------------------------------------

echo "==> [2/3] Removing demo config directory ${DEMO_CONFIG_DIR}..."

if [[ -d "${DEMO_CONFIG_DIR}" ]]; then
    rm -rf "${DEMO_CONFIG_DIR}"
    echo "    Removed ${DEMO_CONFIG_DIR}"
else
    echo "    ${DEMO_CONFIG_DIR} not found — already removed"
fi

# ---------------------------------------------------------------------------
# Step 3: Delete OS keyring entries
# ---------------------------------------------------------------------------

echo "==> [3/3] Deleting demo keyring entries..."
echo "    (Credential names: crowdstrike/client_id, crowdstrike/client_secret,"
echo "     armis/bearer_token, claroty/bearer_token, cyberint/api_key)"

# Use prism-bin credential set mechanism in reverse: delete via platform tools.
# We use the macOS `security` CLI or Linux `secret-tool` as fallback.
# If keyring is unavailable, log a warning but continue (idempotent teardown).

delete_keyring_entry() {
    local sensor="$1"
    local name="$2"
    # Keyring namespace key format: "{org_slug}/{sensor}/{name}" (BC-2.03.004)
    local key="${DEMO_ORG_SLUG}/${sensor}/${name}"

    if [[ "$(uname -s)" == "Darwin" ]]; then
        # macOS: use security CLI to delete the generic password.
        if security delete-generic-password -s "prism" -a "${key}" &>/dev/null; then
            echo "    Deleted (macOS keychain): prism/${sensor}/${name}"
        else
            echo "    SKIP: prism/${sensor}/${name} not found in macOS keychain"
        fi
    else
        # Linux: try secret-tool (libsecret) if available.
        if command -v secret-tool &>/dev/null; then
            if secret-tool clear service "prism" account "${key}" 2>/dev/null; then
                echo "    Deleted (libsecret): prism/${sensor}/${name}"
            else
                echo "    SKIP: prism/${sensor}/${name} not found in libsecret"
            fi
        else
            echo "    WARN: no keyring CLI found (secret-tool not available)." \
                 "Delete manually: service='prism', account='${key}'" >&2
        fi
    fi
}

delete_keyring_entry "crowdstrike" "client_id"
delete_keyring_entry "crowdstrike" "client_secret"
delete_keyring_entry "armis" "bearer_token"
delete_keyring_entry "claroty" "bearer_token"
delete_keyring_entry "cyberint" "api_key"

# Also clean up the credential_index.json if it was created outside the config dir.
if [[ -f "${PRISM_BIN%prism}credential_index.json" ]]; then
    rm -f "${PRISM_BIN%prism}credential_index.json"
fi

echo ""
echo "==> Teardown complete. Demo environment removed."
echo ""
