#!/usr/bin/env bash
# scripts/install.sh — Checksum-verified installer for Prism (macOS + Linux).
#
# USAGE
#   curl -fsSL https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.sh | bash
#   bash install.sh --version v1.0.0-rc.1
#   bash install.sh --version v1.0.0-rc.1 --dry-run
#
# SUPPORTED PLATFORMS
#   aarch64-apple-darwin      macOS (Apple Silicon)
#   x86_64-apple-darwin       macOS (Intel)
#   x86_64-unknown-linux-gnu  Linux (glibc — Debian, Ubuntu, RHEL, etc.)
#   x86_64-unknown-linux-musl Linux (musl — Alpine and other musl-based distros)
#   x86_64-pc-windows-msvc    Windows — NOT supported by this script; use scripts/install.ps1
#
# WHAT IT DOES
#   1. Resolves the latest release via GitHub REST API (includes prereleases, unlike /releases/latest).
#   2. Detects the host platform (composite musl detection per U10).
#   3. Downloads the correct release archive + checksums.txt to a temp dir.
#   4. Verifies the SHA-256 checksum; aborts on mismatch.
#   5. Extracts the prism binary and installs it to INSTALL_DIR.
#   6. Prints PATH guidance if INSTALL_DIR is not in PATH.
#   7. Optionally verifies build provenance via gh attestation verify.
#
# SECURITY
#   - Checksum mismatch aborts install immediately (no silent continuation).
#   - No gh CLI dependency in the script itself (auth-free GitHub REST API for version resolution).
#   - Temp dir is always cleaned up on exit (trap).
#
# Stories: S-REL-003 | ACs: AC-001..AC-009

set -euo pipefail

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------
REPO="drbothen/prism"
INSTALL_DIR="/usr/local/bin"
VERSION=""
DRY_RUN=false
SKIP_VERIFY_PROVENANCE=false

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      if [[ $# -lt 2 ]]; then
        printf 'ERROR: --version requires a value (e.g. --version v1.0.0-rc.1)\n' >&2
        exit 1
      fi
      VERSION="${2}"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --skip-verify-provenance)
      SKIP_VERIFY_PROVENANCE=true
      shift
      ;;
    *)
      printf 'ERROR: unknown argument: %s\n' "$1" >&2
      printf 'Usage: install.sh [--version <tag>] [--dry-run] [--skip-verify-provenance]\n' >&2
      exit 1
      ;;
  esac
done

# ---------------------------------------------------------------------------
# SHA-256 tool detection (U9: macOS ships shasum; Linux ships sha256sum)
# ---------------------------------------------------------------------------
if command -v sha256sum >/dev/null 2>&1; then
  CHECKSUM_CMD=(sha256sum)
elif command -v shasum >/dev/null 2>&1; then
  CHECKSUM_CMD=(shasum -a 256)
else
  printf 'ERROR: neither sha256sum nor shasum found — cannot verify checksums\n' >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Dependencies check
# ---------------------------------------------------------------------------
if ! command -v curl >/dev/null 2>&1; then
  printf 'ERROR: curl is required but not found — install curl and retry\n' >&2
  exit 1
fi
if ! command -v tar >/dev/null 2>&1; then
  printf 'ERROR: tar is required but not found — install tar and retry\n' >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Version resolution
# ---------------------------------------------------------------------------
if [[ -z "${VERSION}" ]]; then
  # U8: /releases/latest EXCLUDES prereleases and drafts (GitHub API docs).
  # Use /releases?per_page=1 instead — returns releases in reverse-chronological
  # order, includes prereleases. No gh CLI dependency; no auth required.
  VERSION="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases?per_page=1" \
    | grep '"tag_name"' | head -1 \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"
  if [[ -z "${VERSION}" ]]; then
    printf 'ERROR: failed to resolve latest release version from GitHub API\n' >&2
    printf '  Try passing an explicit version: --version v1.0.0-rc.1\n' >&2
    exit 1
  fi
fi

# SEC-005: validate VERSION format before URL construction (reject malformed tags)
if [[ ! "${VERSION}" =~ ^v[0-9] ]]; then
  printf 'ERROR: VERSION must start with "v" followed by a digit (e.g. v1.0.0-rc.1); got: %s\n' "${VERSION}" >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Platform detection
# ---------------------------------------------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"

_ldd_musl_check() {
  # Fallback musl indicator (U10): ldd may be absent in busybox containers.
  ldd /bin/sh 2>&1 | grep -q musl 2>/dev/null || return 1
}

case "${OS}-${ARCH}" in
  Darwin-arm64)
    TARGET="aarch64-apple-darwin"
    ;;
  Darwin-x86_64)
    TARGET="x86_64-apple-darwin"
    ;;
  Linux-x86_64)
    # Musl detection (U10): three OR probes — each independently sufficient to identify musl.
    # Logic is OR (not ordered composite): target musl if ANY probe triggers.
    # 1. getconf GNU_LIBC_VERSION — succeeds on glibc, absent on musl (glibc-positive probe).
    # 2. /lib/ld-musl-x86_64.so.1 — filesystem indicator on musl/Alpine x86_64 (musl-positive probe).
    # 3. ldd /bin/sh | grep musl — fallback; unreliable in busybox but serves as advisory.
    # Target musl if ANY musl-positive condition is met (i.e. getconf fails, OR musl path exists,
    # OR ldd grep matches). Logic: OR, not ordered composite.
    if ! getconf GNU_LIBC_VERSION >/dev/null 2>&1 \
        || test -e /lib/ld-musl-x86_64.so.1 \
        || _ldd_musl_check; then
      TARGET="x86_64-unknown-linux-musl"
    else
      TARGET="x86_64-unknown-linux-gnu"
    fi
    ;;
  MINGW*-*|MSYS*-*|CYGWIN*-*|Windows_NT-*)
    # Windows reached via Git Bash, MSYS2, or Cygwin — use the PowerShell installer instead.
    printf 'ERROR: install.sh does not support Windows.\n' >&2
    printf '  Use scripts/install.ps1 for Windows (x86_64-pc-windows-msvc):\n' >&2
    printf '  irm https://raw.githubusercontent.com/drbothen/prism/main/scripts/install.ps1 | iex\n' >&2
    exit 1
    ;;
  *)
    printf 'ERROR: unsupported platform: %s-%s\n' "${OS}" "${ARCH}" >&2
    printf '  Supported: macOS (arm64, x86_64), Linux x86_64 (glibc/musl)\n' >&2
    printf '  For Windows (x86_64-pc-windows-msvc), use scripts/install.ps1\n' >&2
    exit 1
    ;;
esac

# ---------------------------------------------------------------------------
# URL construction
# ---------------------------------------------------------------------------
ARCHIVE="prism-${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE}"
CHECKSUM_URL="https://github.com/${REPO}/releases/download/${VERSION}/checksums.txt"

if [[ "${DRY_RUN}" == "true" ]]; then
  printf 'Dry run — would download:\n'
  printf '  Archive:   %s\n' "${URL}"
  printf '  Checksums: %s\n' "${CHECKSUM_URL}"
  printf '  Target:    %s\n' "${TARGET}"
  printf '  Version:   %s\n' "${VERSION}"
  exit 0
fi

# ---------------------------------------------------------------------------
# Install directory selection
# ---------------------------------------------------------------------------
# Prefer /usr/local/bin; fall back to ~/.local/bin if not writable.
if ! test -w "${INSTALL_DIR}"; then
  INSTALL_DIR="${HOME}/.local/bin"
  mkdir -p "${INSTALL_DIR}"
fi

# ---------------------------------------------------------------------------
# Temporary directory + cleanup trap
# ---------------------------------------------------------------------------
TMPDIR_PRISM="$(mktemp -d)"
_cleanup() { rm -rf "${TMPDIR_PRISM}"; }
trap _cleanup EXIT

# ---------------------------------------------------------------------------
# Download
# ---------------------------------------------------------------------------
printf 'Downloading prism %s for %s...\n' "${VERSION}" "${TARGET}"
curl -fsSL --output "${TMPDIR_PRISM}/${ARCHIVE}" "${URL}"
curl -fsSL --output "${TMPDIR_PRISM}/checksums.txt" "${CHECKSUM_URL}"

# ---------------------------------------------------------------------------
# SHA-256 verification (AC-003: abort on mismatch)
# ---------------------------------------------------------------------------
EXPECTED="$(grep -F -- "${ARCHIVE}" "${TMPDIR_PRISM}/checksums.txt" | awk '{print $1}')"
if [[ -z "${EXPECTED}" ]]; then
  printf 'ERROR: %s not found in checksums.txt\n' "${ARCHIVE}" >&2
  exit 1
fi

ACTUAL="$("${CHECKSUM_CMD[@]}" "${TMPDIR_PRISM}/${ARCHIVE}" | awk '{print $1}')"

if [[ "${EXPECTED}" != "${ACTUAL}" ]]; then
  printf 'ERROR: Checksum mismatch for %s\n' "${ARCHIVE}" >&2
  printf '  Expected: %s\n' "${EXPECTED}" >&2
  printf '  Actual:   %s\n' "${ACTUAL}" >&2
  exit 1
fi
printf 'Checksum verified.\n'

# ---------------------------------------------------------------------------
# Optional provenance verification (requires gh CLI)
# ---------------------------------------------------------------------------
if [[ "${SKIP_VERIFY_PROVENANCE}" != "true" ]] && command -v gh >/dev/null 2>&1; then
  printf 'Verifying build provenance...\n'
  if ! gh attestation verify "${TMPDIR_PRISM}/${ARCHIVE}" \
      --repo "${REPO}" \
      --signer-workflow "${REPO}/.github/workflows/release.yml" 2>/dev/null; then
    printf 'ERROR: provenance verification failed for %s\n' "${ARCHIVE}" >&2
    printf '  The archive could not be verified against the GitHub Actions build workflow.\n' >&2
    printf '  To skip provenance check, pass --skip-verify-provenance\n' >&2
    exit 1
  else
    printf 'Build provenance verified.\n'
  fi
fi

# ---------------------------------------------------------------------------
# Extract and install
# ---------------------------------------------------------------------------
tar -xzf "${TMPDIR_PRISM}/${ARCHIVE}" -C "${TMPDIR_PRISM}" prism
cp "${TMPDIR_PRISM}/prism" "${INSTALL_DIR}/prism"
chmod 755 "${INSTALL_DIR}/prism"

# ---------------------------------------------------------------------------
# PATH guidance (AC-004)
# ---------------------------------------------------------------------------
case ":${PATH}:" in
  *":${INSTALL_DIR}:"*)
    : # already in PATH
    ;;
  *)
    printf '\nNOTE: %s is not in your PATH.\n' "${INSTALL_DIR}"
    printf '  Add it with:\n'
    printf "    export PATH=\"%s:\$PATH\"\n" "${INSTALL_DIR}"
    printf '  (Add this line to your shell profile: ~/.bashrc, ~/.zshrc, etc.)\n'
    ;;
esac

# ---------------------------------------------------------------------------
# Confirm install
# ---------------------------------------------------------------------------
if INSTALLED_VERSION="$("${INSTALL_DIR}/prism" --version 2>/dev/null)"; then
  printf 'prism installed to %s/prism (%s)\n' "${INSTALL_DIR}" "${INSTALLED_VERSION}"
else
  printf 'prism installed to %s/prism (version: %s)\n' "${INSTALL_DIR}" "${VERSION}"
fi

# ---------------------------------------------------------------------------
# Post-install notice (binary-only install; specs ship via demo bundle)
# ---------------------------------------------------------------------------
printf '\nNOTE: This installer deploys the prism binary only (binary-only install is intentional).\n'
printf '  Configuration: obtain prism.toml.example from the repository or demo bundle:\n'
printf '    https://github.com/%s/blob/main/prism.toml.example\n' "${REPO}"
printf '  Sensor specs:  see RELEASING.md or the forthcoming demo bundle for sensor spec files.\n'
printf '  See RELEASING.md for the full post-install setup guide.\n'
