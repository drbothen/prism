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
