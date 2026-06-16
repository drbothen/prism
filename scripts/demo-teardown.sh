#!/usr/bin/env bash
# scripts/demo-teardown.sh — Remove the Prism DTU demo environment.
#
# USAGE
#   bash scripts/demo-teardown.sh [--config-dir DIR]
#
# WHAT IT DOES
#   1. Kills the DTU demo server (via single PID file — start-multi uses one process)
#   2. Deletes the 10 demo OS keyring entries via `prism credential delete` (F-P10-HIGH-001)
#      N=3 orgs × M=variable sensors = 10 credential entries total:
#        org-a: crowdstrike/{client_id,client_secret}, armis/bearer_token
#        org-b: claroty/bearer_token, cyberint/api_key
#        org-c: crowdstrike/{client_id,client_secret}, armis/bearer_token,
#               claroty/bearer_token, cyberint/api_key
#   3. Removes <DIR>/ (config, specs, plugins, state)
#   4. Exits 0
#
# NOTE: keyring deletes (Step 2) run BEFORE config dir removal (Step 3) because
# `prism credential delete` reads prism.toml to resolve the OrgId UUID for the
# OrgId-keyed namespace (ADR-034 §D3). If the config dir were removed first,
# prism.toml would be unavailable → all deletes would fail silently.
#
# EC-005: If no PID file is found, prints "server may not be running", skips kill,
#         and continues to credential delete + rm -rf (idempotent teardown).
#
# AC-007: DTU server is killed; config dir removed; keyring entries deleted.
# AC-008: passes shellcheck with zero errors/warnings.
#
# Stories: S-DEMO-003 | S-DEMO-LAUNCHER-CONSOLIDATION-001 | BC: BC-2.03.005

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
PRISM_TOML_PATH="${DEMO_CONFIG_DIR}/prism.toml"

# ---------------------------------------------------------------------------
# Resolve per-org IDs from prism.toml — required for OrgId-keyed keyring delete.
#
# The keyring entries were written by demo-setup.sh via `prism credential set`
# which uses CredentialStoreOrgId::set_by_org — namespace format:
#   "{org_id_uuid}/{sensor}/{name}"  (namespace_key_by_org_id; ADR-034 §D3).
#
# We must use the SAME org slugs that were used during setup so that
# `prism credential delete --org-slug <slug>` can resolve the OrgId UUID
# from prism.toml (which lists all 3 [[orgs]] entries).
#
# IMPORTANT: keyring deletes run BEFORE the `rm -rf` of the config dir (Step 3).
#
# If prism.toml is absent, we skip the keyring delete with a warning
# (idempotent teardown: entries may already be absent or were never set).
# ---------------------------------------------------------------------------

PRISM_TOML_PRESENT=false
if [[ -f "${PRISM_TOML_PATH}" ]]; then
    PRISM_TOML_PRESENT=true
else
    echo "    WARN: ${PRISM_TOML_PATH} not found — keyring entries will be skipped" >&2
fi

# ---------------------------------------------------------------------------
# Step 1: Kill DTU demo server (single start-multi process)
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
    # EC-005: no PID file — server may not be running; continue teardown idempotently.
    echo "    No DTU PID file found — server may not be running"
fi

# ---------------------------------------------------------------------------
# Step 2: Delete OS keyring entries (N×M = 10 total)
#
# F-P10-HIGH-001 fix: keyring deletes MUST run BEFORE `rm -rf` of the config
# dir (Step 3), because `prism credential delete` needs prism.toml to resolve
# the OrgId UUID for the OrgId-keyed namespace. Previous ordering (delete AFTER
# rm -rf) made prism.toml unavailable → credential deletes always failed silently.
# ---------------------------------------------------------------------------

echo "==> [2/3] Deleting demo keyring entries (10 total across 3 orgs)..."

# F-P10-HIGH-001 fix: use `prism credential delete` on ALL platforms (macOS, Linux, Windows).
#
# RATIONALE: `prism credential delete` calls delete_by_org (CredentialStoreOrgId::delete_by_org)
# which uses the EXACT same namespace + backend attribute schema as the write path (set_by_org).
# This is guaranteed to match on all platforms by construction. Platform-native CLI tools
# (security delete-generic-password, secret-tool clear) use different attribute schemas
# and would silently miss the entries (F-P10-HIGH-001 root cause in S-DEMO-003).
#
# Exit 0 = deleted OR already absent (idempotent). Exit 1 = backend error (warn, continue).

delete_keyring_entry() {
    local org_slug="$1"
    local sensor="$2"
    local name="$3"
    if "${PRISM_BIN}" credential delete \
            --sensor "${sensor}" \
            --name "${name}" \
            --org-slug "${org_slug}" \
            --config-dir "${DEMO_CONFIG_DIR}" 2>/dev/null; then
        echo "    Deleted keyring: ${org_slug}/${sensor}/${name}"
    else
        local rc=$?
        echo "    WARN: could not delete ${org_slug}/${sensor}/${name} from keyring (prism exit ${rc})" >&2
    fi
}

if [[ "${PRISM_TOML_PRESENT}" == "true" ]] && [[ -x "${PRISM_BIN}" ]]; then
    # org-a: 3 credentials
    delete_keyring_entry "org-a" "crowdstrike" "client_id"
    delete_keyring_entry "org-a" "crowdstrike" "client_secret"
    delete_keyring_entry "org-a" "armis"       "bearer_token"

    # org-b: 2 credentials
    delete_keyring_entry "org-b" "claroty"  "bearer_token"
    delete_keyring_entry "org-b" "cyberint" "api_key"

    # org-c: 5 credentials
    delete_keyring_entry "org-c" "crowdstrike" "client_id"
    delete_keyring_entry "org-c" "crowdstrike" "client_secret"
    delete_keyring_entry "org-c" "armis"       "bearer_token"
    delete_keyring_entry "org-c" "claroty"     "bearer_token"
    delete_keyring_entry "org-c" "cyberint"    "api_key"
elif [[ "${PRISM_TOML_PRESENT}" == "false" ]]; then
    echo "    SKIP: prism.toml not found — keyring entries not deleted." >&2
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
