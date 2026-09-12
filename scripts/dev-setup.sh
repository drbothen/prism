#!/usr/bin/env bash
# scripts/dev-setup.sh — install and configure the Prism development toolchain.
# Idempotent: safe to run multiple times. Each tool is only installed if absent.
set -euo pipefail

# Verify rustup is available (prerequisite for cargo tool installs)
if ! command -v rustup >/dev/null 2>&1; then
  echo "ERROR: rustup not found. Install from https://rustup.rs before running this script."
  exit 1
fi
echo "  ✓ rustup present ($(rustup --version 2>&1 | head -1))"

# protoc is the one required tool that is not a cargo install: crates/prism-ocsf's
# build script shells out to it via prost-build. prism-ocsf sits early in the
# dependency graph, so without protoc every workspace-wide cargo command aborts in
# that build script, including ones unrelated to OCSF. Checked first, before the
# slow cargo installs, so the failure is immediate rather than minutes into a build.
ensure_protoc() {
  if command -v protoc >/dev/null 2>&1; then
    echo "  ✓ protoc present ($(protoc --version 2>&1 | head -1))"
    return 0
  fi
  if [ -n "${PROTOC:-}" ] && [ -x "${PROTOC}" ]; then
    echo "  ✓ protoc present via PROTOC=${PROTOC}"
    return 0
  fi

  echo "  → installing protoc"
  if command -v brew >/dev/null 2>&1; then
    brew install protobuf
  elif command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update && sudo apt-get install -y protobuf-compiler
  elif command -v dnf >/dev/null 2>&1; then
    sudo dnf install -y protobuf-compiler
  elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -S --needed --noconfirm protobuf
  elif command -v apk >/dev/null 2>&1; then
    sudo apk add protobuf
  else
    echo "ERROR: protoc is required by crates/prism-ocsf and no supported package"
    echo "       manager was found (brew, apt-get, dnf, pacman, apk)."
    echo "       Install it manually, then re-run this script:"
    echo "         Windows: winget install protobuf   (or: choco install protoc)"
    echo "         Any OS:  https://github.com/protocolbuffers/protobuf/releases"
    echo "       Already have it somewhere else? Export PROTOC=/path/to/protoc."
    exit 1
  fi

  if ! command -v protoc >/dev/null 2>&1; then
    echo "ERROR: protoc still not on PATH after install. Open a new shell, or export"
    echo "       PROTOC=/path/to/protoc, then re-run this script."
    exit 1
  fi
  echo "  ✓ protoc installed ($(protoc --version 2>&1 | head -1))"
}

ensure_protoc

# Helper: install a cargo binary only if it is not already on PATH.
install_if_missing() {
  local tool="$1"
  local pkg="${2:-$1}"
  if command -v "$tool" >/dev/null 2>&1; then
    echo "  ✓ $tool already installed"
  else
    echo "  → installing $pkg"
    cargo install --locked "$pkg"
  fi
}

echo ""
echo "Installing cargo tool extensions..."

install_if_missing cargo-deny
install_if_missing cargo-audit
install_if_missing cargo-semver-checks
install_if_missing cargo-mutants
install_if_missing cargo-fuzz
install_if_missing cargo-llvm-cov

# kani-verifier installs as `kani` on PATH but the package is kani-verifier
install_if_missing kani kani-verifier

# just and lefthook are system tools; install via cargo if not present
install_if_missing just
install_if_missing lefthook

echo ""
echo "Configuring git hooks via lefthook..."
lefthook install

echo ""
echo "Development toolchain ready"
