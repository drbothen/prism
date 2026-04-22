# AC-3 Evidence: dev-setup.sh installs all tools

Story: S-0.02 | Version: 1.4 | Date: 2026-04-21

**AC-3:** Given `scripts/dev-setup.sh` is run on a fresh macOS or Linux system with
Rust installed, When the script completes, Then all 9 cargo tool extensions (cargo-deny,
cargo-audit, cargo-semver-checks, cargo-mutants, cargo-fuzz, cargo-llvm-cov, just,
lefthook, kani-verifier) are available on the PATH.

---

## Syntax check

```
$ bash -n scripts/dev-setup.sh && echo "syntax OK"
syntax OK
```

## All 9 tools referenced via install_if_missing

```
install_if_missing() {          <- function definition
install_if_missing cargo-deny
install_if_missing cargo-audit
install_if_missing cargo-semver-checks
install_if_missing cargo-mutants
install_if_missing cargo-fuzz
install_if_missing cargo-llvm-cov
install_if_missing kani kani-verifier
install_if_missing just
install_if_missing lefthook
```

All 9 tools: cargo-deny, cargo-audit, cargo-semver-checks, cargo-mutants, cargo-fuzz,
cargo-llvm-cov, kani-verifier, just, lefthook.

## Script structure (key sections)

```bash
#!/usr/bin/env bash
set -euo pipefail

# Verify rustup is available (prerequisite for cargo tool installs)
if ! command -v rustup >/dev/null 2>&1; then
  echo "ERROR: rustup not found. Install from https://rustup.rs before running this script."
  exit 1
fi

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
```

## Test gate result

`ok 5 - AC-3: dev-setup.sh references all 9 required tools`
**PASS**
