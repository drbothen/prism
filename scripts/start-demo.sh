#!/usr/bin/env bash
# scripts/start-demo.sh — convenience launcher for prism-dtu-demo-server
#
# Builds (if needed) and starts the multi-clone DTU demo harness.
# Defaults to configs/demo.toml (plain HTTP, all 6 clones on ports 17080-17085).
#
# Usage:
#   scripts/start-demo.sh [OPTIONS]
#
# Options:
#   --config <PATH>            Path to demo TOML config (default: crates/prism-dtu-demo-server/configs/demo.toml)
#   --tls                      Enable HTTPS (binary must be built with --features tls)
#   --bind-any                 Allow non-loopback binding (R-DEMO-001: also requires
#                                PRISM_DTU_DEMO_ALLOW_NETWORK_BIND=I-UNDERSTAND-THE-RISK)
#   --deterministic-logging    Suppress timestamps/PIDs for AC-7 determinism
#   --help                     Print this message and exit
#
# Environment:
#   CONFIG                     Override --config (lower priority than --config flag)
#
# Examples:
#   scripts/start-demo.sh                                            # plain HTTP, default config
#   scripts/start-demo.sh --tls                                      # HTTPS with self-signed cert
#   scripts/start-demo.sh --config configs/prism-demo.toml --deterministic-logging

set -euo pipefail

# ---------------------------------------------------------------------------
# Resolve project root (one level up from this script's directory)
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ---------------------------------------------------------------------------
# Defaults
# ---------------------------------------------------------------------------
CONFIG="${CONFIG:-${PROJECT_ROOT}/crates/prism-dtu-demo-server/configs/demo.toml}"
EXTRA_ARGS=()

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
    case "$1" in
        --config)
            if [[ -z "${2:-}" ]]; then
                echo "error: --config requires a PATH argument" >&2
                exit 1
            fi
            CONFIG="$2"
            shift 2
            ;;
        --tls|--bind-any|--deterministic-logging)
            EXTRA_ARGS+=("$1")
            shift
            ;;
        --help|-h)
            # Print the leading comment block (lines 2-N that start with '#'),
            # stopping at the first line that does NOT start with '#'.
            awk 'NR==1{next} /^#/{print substr($0,3)} /^[^#]/{exit}' "${BASH_SOURCE[0]}"
            exit 0
            ;;
        *)
            echo "error: unknown argument: $1" >&2
            echo "Run '$0 --help' for usage." >&2
            exit 1
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Build features
# ---------------------------------------------------------------------------
BUILD_FEATURES="dtu,tls"
BINARY="${PROJECT_ROOT}/target/release/prism-dtu-demo-server"

echo "==> Building prism-dtu-demo-server (--features ${BUILD_FEATURES})..."
cargo build --release -p prism-dtu-demo-server --features "${BUILD_FEATURES}" \
    --manifest-path "${PROJECT_ROOT}/Cargo.toml"

# ---------------------------------------------------------------------------
# Startup info
# ---------------------------------------------------------------------------
echo ""
echo "==> Starting demo harness"
echo "    Config : ${CONFIG}"
echo "    Binary : ${BINARY}"
if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
    echo "    Flags  : ${EXTRA_ARGS[*]}"
fi
echo ""
echo "    PID file  : ${PROJECT_ROOT}/.prism-dtu-demo-server.pid"
echo "    URL sidecar: ${PROJECT_ROOT}/.prism-dtu-demo-server.urls.json"
echo ""
echo "    Stop with: prism-dtu-demo-server stop"
echo "              OR: kill \$(cat .prism-dtu-demo-server.pid)"
echo ""

# ---------------------------------------------------------------------------
# Demo credential environment (S-6.20 Task 11 / IMPORTANT-001 closure)
#
# configs/prism-demo.toml uses bare-name credential_ref values such as
# DEMO_FAKE_CROWDSTRIKE_TOKEN.  The S-5.05 resolver checks:
#   1. <NAME>_FILE env var  (path to a file containing the secret)
#   2. <NAME> env var       (the secret value itself)  <-- these exports hit tier 2
#   3. platform keyring
#
# These are FAKE tokens accepted by the DTU clones' fixture validators.
# They are NOT real credentials and must NEVER be replaced with real secrets
# here.  If you need a different value, export the variable before calling
# this script — the "${VAR:-default}" pattern lets your export win.
#
# DO NOT commit real tokens to this file.
# ---------------------------------------------------------------------------
export DEMO_FAKE_CROWDSTRIKE_TOKEN="${DEMO_FAKE_CROWDSTRIKE_TOKEN:-dtu-fake-cs-token}"
export DEMO_FAKE_CLAROTY_TOKEN="${DEMO_FAKE_CLAROTY_TOKEN:-dtu-fake-claroty-token}"
export DEMO_FAKE_CYBERINT_TOKEN="${DEMO_FAKE_CYBERINT_TOKEN:-dtu-fake-cyberint-token}"
export DEMO_FAKE_ARMIS_TOKEN="${DEMO_FAKE_ARMIS_TOKEN:-dtu-fake-armis-token}"
export DEMO_FAKE_THREATINTEL_TOKEN="${DEMO_FAKE_THREATINTEL_TOKEN:-dtu-fake-ti-token}"
export DEMO_FAKE_NVD_TOKEN="${DEMO_FAKE_NVD_TOKEN:-dtu-fake-nvd-token}"

# ---------------------------------------------------------------------------
# Exec the binary — replaces this shell process
# ---------------------------------------------------------------------------
exec "${BINARY}" start --config "${CONFIG}" "${EXTRA_ARGS[@]}"
