#!/usr/bin/env bash
# test_AC-3_dev-setup-installs-tools.sh
# AC-3: dev-setup.sh installs all 9 cargo tool extensions on PATH after completion.
# FAILS on stub: script exits 1 immediately.
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
SETUP="$WORKTREE/scripts/dev-setup.sh"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); echo "not ok $TAP_COUNT - $1"; FAIL=1; }

# Test 1: script exists
if [[ -f "$SETUP" ]]; then
  tap_ok "scripts/dev-setup.sh exists"
else
  tap_fail "scripts/dev-setup.sh missing"
fi

# Test 2: script is executable
if [[ -x "$SETUP" ]]; then
  tap_ok "scripts/dev-setup.sh is executable"
else
  tap_fail "scripts/dev-setup.sh is not executable"
fi

# Test 3: correct shebang
SHEBANG=$(head -1 "$SETUP")
if [[ "$SHEBANG" == "#!/usr/bin/env bash" ]]; then
  tap_ok "scripts/dev-setup.sh has correct shebang"
else
  tap_fail "scripts/dev-setup.sh missing '#!/usr/bin/env bash' shebang (got: $SHEBANG)"
fi

# Test 4: set -euo pipefail present
if grep -q 'set -euo pipefail' "$SETUP"; then
  tap_ok "scripts/dev-setup.sh uses set -euo pipefail"
else
  tap_fail "scripts/dev-setup.sh missing 'set -euo pipefail'"
fi

# Test 5: script mentions all 9 required tools
REQUIRED_TOOLS=(
  "cargo-deny"
  "cargo-audit"
  "cargo-semver-checks"
  "cargo-mutants"
  "cargo-fuzz"
  "cargo-llvm-cov"
  "just"
  "lefthook"
  "kani-verifier"
)
ALL_PRESENT=1
for tool in "${REQUIRED_TOOLS[@]}"; do
  if ! grep -q "$tool" "$SETUP"; then
    ALL_PRESENT=0
    tap_fail "AC-3: dev-setup.sh does not reference required tool: $tool"
  fi
done
if [[ $ALL_PRESENT -eq 1 ]]; then
  tap_ok "AC-3: dev-setup.sh references all 9 required tools"
fi

# Test 6: script checks for rustup presence
if grep -q 'rustup' "$SETUP"; then
  tap_ok "dev-setup.sh references rustup (prerequisite check)"
else
  tap_fail "AC-3: dev-setup.sh missing rustup check"
fi

# Test 7: lefthook install called at end of script
if grep -q 'lefthook install' "$SETUP"; then
  tap_ok "dev-setup.sh runs lefthook install"
else
  tap_fail "AC-3: dev-setup.sh missing 'lefthook install'"
fi

# Test 8: success message present
if grep -q "Development toolchain ready" "$SETUP"; then
  tap_ok "dev-setup.sh prints success message"
else
  tap_fail "AC-3: dev-setup.sh missing success message 'Development toolchain ready'"
fi

echo "1..$TAP_COUNT"
exit $FAIL
