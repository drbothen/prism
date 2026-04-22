#!/usr/bin/env bash
# test_AC-5_deny-toml-license-check.sh
# AC-5: `cargo deny check` enforces license allowlist from deny.toml.
# FAILS on stub: cargo-deny not installed (dev-setup.sh hasn't run), deny.toml is placeholder.
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
DENY_TOML="$WORKTREE/deny.toml"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); echo "not ok $TAP_COUNT - $1"; FAIL=1; }
tap_skip() { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - SKIP $1 # SKIP tool absent"; }

# Test 1: deny.toml exists
if [[ -f "$DENY_TOML" ]]; then
  tap_ok "deny.toml exists"
else
  tap_fail "deny.toml missing"
fi

# Test 2: [licenses] section with allow list present
if grep -q '^\[licenses\]' "$DENY_TOML"; then
  tap_ok "deny.toml has [licenses] section"
else
  tap_fail "AC-5: deny.toml missing [licenses] section"
fi

# Test 3: required OSI-approved licenses in allowlist
REQUIRED_LICENSES=("MIT" "Apache-2.0" "BSD-2-Clause" "BSD-3-Clause" "ISC" "Unicode-DFS-2016" "Zlib")
ALL_OK=1
for lic in "${REQUIRED_LICENSES[@]}"; do
  if ! grep -qF "\"$lic\"" "$DENY_TOML"; then
    ALL_OK=0
    tap_fail "AC-5: deny.toml allowlist missing license: $lic"
  fi
done
if [[ $ALL_OK -eq 1 ]]; then
  tap_ok "AC-5: deny.toml allowlist contains all required OSI-approved licenses"
fi

# Test 4: [advisories] with vulnerability = "deny"
if grep -q 'vulnerability = "deny"' "$DENY_TOML"; then
  tap_ok "deny.toml sets vulnerability = \"deny\""
else
  tap_fail "AC-5: deny.toml missing vulnerability = \"deny\" in [advisories]"
fi

# Test 5: [bans] wildcards = "deny"
if grep -q 'wildcards = "deny"' "$DENY_TOML"; then
  tap_ok "deny.toml sets wildcards = \"deny\""
else
  tap_fail "AC-5: deny.toml missing wildcards = \"deny\" (architecture compliance rule)"
fi

# Test 6: [sources] unknown-registry = "deny"
if grep -q 'unknown-registry = "deny"' "$DENY_TOML"; then
  tap_ok "deny.toml sets unknown-registry = \"deny\""
else
  tap_fail "AC-5: deny.toml missing unknown-registry = \"deny\""
fi

# Test 7: `cargo deny check` actually runs (requires cargo-deny installed)
if ! command -v cargo-deny &>/dev/null && ! cargo deny --version &>/dev/null 2>&1; then
  tap_skip "cargo-deny not on PATH — run scripts/dev-setup.sh first"
else
  cd "$WORKTREE"
  if cargo deny check --config "$DENY_TOML" &>/dev/null; then
    tap_ok "AC-5: cargo deny check passes with deny.toml"
  else
    # On a workspace with no deps, deny check should pass; fail means config issue
    tap_fail "AC-5: cargo deny check failed — deny.toml may have configuration errors"
  fi
fi

echo "1..$TAP_COUNT"
exit $FAIL
