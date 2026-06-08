#!/usr/bin/env bash
# scripts/demo-teardown.sh — Remove the Prism DTU demo environment.
#
# USAGE
#   bash scripts/demo-teardown.sh [--config-dir DIR]
#
# WHAT IT DOES
#   1. Kills the DTU demo server (via PID file)
#   2. Deletes the 5 demo OS keyring entries via `prism credential delete` (F-P10-HIGH-001)
#   3. Removes ~/.config/prism-demo/ (config, specs, plugins, state)
#   4. Exits 0
#
# NOTE: keyring deletes (Step 2) run BEFORE config dir removal (Step 3) because
# `prism credential delete` reads prism.toml to resolve the OrgId UUID for the
# OrgId-keyed namespace (ADR-034 §D3). If the config dir were removed first,
# prism.toml would be unavailable → all deletes would fail.
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
# Resolve DEMO_ORG_ID from prism.toml — required for OrgId-keyed keyring delete.
#
# The keyring entries were written by demo-setup.sh via `prism credential set`
# which uses CredentialStoreOrgId::set_by_org — namespace format:
#   "{org_id_uuid}/{sensor}/{name}"  (namespace_key_by_org_id; ADR-034 §D3).
#
# The slug-keyed format "{org_slug}/{sensor}/{name}" is DISJOINT from the OrgId-
# keyed format — deleting under the slug key would silently miss all 5 entries
# (AC-007 regression). We must use the same OrgId UUID that was used during setup.
#
# IMPORTANT: keyring deletes run BEFORE the `rm -rf` of the config dir (Step 2),
# so prism.toml is still available here for parsing.
#
# If prism.toml is absent or has no org_id, we skip the keyring delete with a
# warning (idempotent teardown: entries may already be absent or were never set).
# ---------------------------------------------------------------------------

DEMO_ORG_ID=""
PRISM_TOML_PATH="${DEMO_CONFIG_DIR}/prism.toml"

if [[ -f "${PRISM_TOML_PATH}" ]]; then
    # Extract org_id from the first [[orgs]] entry.
    # TOML format: org_id = "0196f4b2-3c8d-7e1a-b5f0-2d4c6e8a0b1c"
    DEMO_ORG_ID="$(grep -E '^\s*org_id\s*=' "${PRISM_TOML_PATH}" | head -1 | \
        sed 's/.*=\s*"\([^"]*\)".*/\1/')"
    if [[ -z "${DEMO_ORG_ID}" ]]; then
        echo "    WARN: could not parse org_id from ${PRISM_TOML_PATH}" \
             "— keyring entries will be skipped" >&2
    fi
else
    echo "    WARN: ${PRISM_TOML_PATH} not found — keyring entries will be skipped" >&2
fi

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
# Step 2: Delete OS keyring entries
#
# F-P10-HIGH-001 fix: keyring deletes MUST run BEFORE `rm -rf` of the config
# dir (Step 3), because `prism credential delete` needs prism.toml to resolve
# the OrgId UUID for the OrgId-keyed namespace. Previous ordering (delete AFTER
# rm -rf) made prism.toml unavailable → credential deletes always failed silently.
# ---------------------------------------------------------------------------

echo "==> [2/3] Deleting demo keyring entries..."
echo "    (Credential names: crowdstrike/client_id, crowdstrike/client_secret,"
echo "     armis/bearer_token, claroty/bearer_token, cyberint/api_key)"

# F-P10-HIGH-001 fix: use `prism credential delete` on ALL platforms (macOS, Linux, Windows).
#
# RATIONALE: Platform-native CLI tools (security delete-generic-password on macOS,
# secret-tool clear on Linux) have different attribute schemas than keyring-rs 3.x:
#   - Linux: secret-tool clear used `account="${key}"` but keyring-rs dbus-secret-service
#     stores the user under the `username` attribute — the clear matched nothing → 5
#     orphaned keyring entries after every teardown (F-P10-HIGH-001 root cause).
#   - macOS: security CLI was correct, but we unify to one path for consistency.
#
# `prism credential delete` calls delete_by_org (CredentialStoreOrgId::delete_by_org)
# which uses the EXACT same namespace + backend attribute schema as the write path
# (set_by_org). This is guaranteed to match on all platforms by construction.
#
# Exit 0 = deleted OR already absent (idempotent). Exit 1 = backend error (warn, continue).

delete_keyring_entry() {
    local sensor="$1"
    local name="$2"
    if "${PRISM_BIN}" credential delete \
            --sensor "${sensor}" \
            --name "${name}" \
            --config-dir "${DEMO_CONFIG_DIR}" 2>/dev/null; then
        echo "    Deleted keyring: ${sensor}/${name}"
    else
        local rc=$?
        echo "    WARN: could not delete ${sensor}/${name} from keyring (prism exit ${rc})" >&2
    fi
}

if [[ -n "${DEMO_ORG_ID}" ]] && [[ -x "${PRISM_BIN}" ]]; then
    delete_keyring_entry "crowdstrike" "client_id"
    delete_keyring_entry "crowdstrike" "client_secret"
    delete_keyring_entry "armis" "bearer_token"
    delete_keyring_entry "claroty" "bearer_token"
    delete_keyring_entry "cyberint" "api_key"
elif [[ -z "${DEMO_ORG_ID}" ]]; then
    echo "    SKIP: DEMO_ORG_ID could not be determined — keyring entries not deleted." >&2
    echo "    To delete manually, find entries with service='prism' in your OS keyring." >&2
else
    echo "    SKIP: prism binary not found at ${PRISM_BIN} — build first with: cargo build --release" >&2
fi

# ---------------------------------------------------------------------------
# Step 3: Remove config directory
#
# Runs AFTER keyring deletes (Step 2) so prism.toml is available for OrgId
# resolution during delete. Previous ordering had this at Step 2.
# ---------------------------------------------------------------------------

echo "==> [3/3] Removing demo config directory ${DEMO_CONFIG_DIR}..."

if [[ -d "${DEMO_CONFIG_DIR}" ]]; then
    rm -rf "${DEMO_CONFIG_DIR}"
    echo "    Removed ${DEMO_CONFIG_DIR}"
else
    echo "    ${DEMO_CONFIG_DIR} not found — already removed"
fi

echo ""
echo "==> Teardown complete. Demo environment removed."
echo ""
