#!/usr/bin/env bash
# test_AC-6_semgrep-credential-rule.sh
# AC-6: Semgrep rule `prism-no-string-credentials` fires on `let api_key: String = get_secret()`.
# FAILS on stub: placeholder patterns match literal "TODO", not the real credential pattern.
set -euo pipefail

WORKTREE="$(cd "$(dirname "$0")/../.." && pwd)"
SEMGREP_DIR="$WORKTREE/.semgrep"
TAP_COUNT=0
FAIL=0

tap_ok()   { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - $1"; }
tap_fail() { TAP_COUNT=$((TAP_COUNT+1)); echo "not ok $TAP_COUNT - $1"; FAIL=1; }
tap_skip() { TAP_COUNT=$((TAP_COUNT+1)); echo "ok $TAP_COUNT - SKIP $1 # SKIP tool absent"; }

# Test 1: .semgrep/ directory exists
if [[ -d "$SEMGREP_DIR" ]]; then
  tap_ok ".semgrep/ directory exists"
else
  tap_fail ".semgrep/ directory missing"
fi

# Test 2: credential-handling.yml exists
if [[ -f "$SEMGREP_DIR/credential-handling.yml" ]]; then
  tap_ok ".semgrep/credential-handling.yml exists"
else
  tap_fail ".semgrep/credential-handling.yml missing"
fi

# Test 3: unsafe-patterns.yml exists
if [[ -f "$SEMGREP_DIR/unsafe-patterns.yml" ]]; then
  tap_ok ".semgrep/unsafe-patterns.yml exists"
else
  tap_fail ".semgrep/unsafe-patterns.yml missing"
fi

# Test 4: rule ID prism-no-string-credentials is declared
if grep -q 'prism-no-string-credentials' "$SEMGREP_DIR/credential-handling.yml"; then
  tap_ok "prism-no-string-credentials rule declared"
else
  tap_fail "AC-6: prism-no-string-credentials rule missing from credential-handling.yml"
fi

# Test 5: rule severity is ERROR (not WARN)
# Search the full file — severity may appear several lines after the id
if grep -A10 'prism-no-string-credentials' "$SEMGREP_DIR/credential-handling.yml" | grep -q 'severity: ERROR'; then
  tap_ok "prism-no-string-credentials has severity: ERROR"
else
  tap_fail "AC-6: prism-no-string-credentials severity is not ERROR"
fi

# Test 6: stub rule patterns must NOT contain "TODO" post-implementation.
# Currently they do — Red Gate: rule is not yet implemented.
if grep -q 'pattern: "TODO"' "$SEMGREP_DIR/credential-handling.yml"; then
  tap_fail "AC-6: credential-handling.yml still has TODO placeholder patterns — real pattern not implemented (Red Gate)"
else
  tap_ok "AC-6: credential-handling.yml has real patterns (no TODO placeholders)"
fi

# Test 7: prism-no-log-secret rule declared
if grep -q 'prism-no-log-secret' "$SEMGREP_DIR/credential-handling.yml"; then
  tap_ok "prism-no-log-secret rule declared"
else
  tap_fail "AC-6: prism-no-log-secret rule missing from credential-handling.yml"
fi

# Test 8: unsafe-patterns.yml has prism-unsafe-block rule
if grep -q 'prism-unsafe-block' "$SEMGREP_DIR/unsafe-patterns.yml"; then
  tap_ok "prism-unsafe-block rule declared"
else
  tap_fail "AC-6: prism-unsafe-block rule missing from unsafe-patterns.yml"
fi

# Test 9: semgrep --validate catches bad patterns (stub has "pattern: TODO" which is invalid Rust semgrep)
if ! command -v semgrep &>/dev/null; then
  tap_skip "semgrep not on PATH — cannot run semgrep --validate"
else
  # Stub rules with `pattern: "TODO"` should fail validation for Rust language
  TMPDIR_TEST=$(mktemp -d)
  # Write a trivially correct Rust test case file
  cat > "$TMPDIR_TEST/test.rs" <<'RUST'
fn main() {
    let api_key: String = get_secret();
}
RUST
  # Running semgrep on the stub rules against a real Rust file:
  # The stub pattern "TODO" will not match, so 0 findings — the rule FAILS to fire.
  # AC-6 requires the rule to FIRE on this input. So: stub failing to fire = Red Gate.
  SEMGREP_OUT=$(semgrep --config "$SEMGREP_DIR/credential-handling.yml" "$TMPDIR_TEST/test.rs" --json 2>/dev/null || true)
  MATCH_COUNT=0
  # Semgrep 1.x produces compact JSON with path-prefixed check_ids; match on rule name substring only.
  if echo "$SEMGREP_OUT" | grep -q 'prism-no-string-credentials' 2>/dev/null; then
    MATCH_COUNT=1
  fi
  # Optional: if jq is available, perform a structured assertion as a second check.
  if [[ "$MATCH_COUNT" -gt 0 ]] && command -v jq &>/dev/null; then
    JQ_COUNT=$(echo "$SEMGREP_OUT" | jq '[.results[]?.check_id // "" | test("prism-no-string-credentials")] | map(select(.)) | length' 2>/dev/null || echo 0)
    if [[ "$JQ_COUNT" -gt 0 ]]; then
      MATCH_COUNT=$JQ_COUNT
    fi
  fi
  rm -rf "$TMPDIR_TEST"

  if [[ "$MATCH_COUNT" -gt 0 ]]; then
    tap_ok "AC-6: prism-no-string-credentials fires on String credential code"
  else
    tap_fail "AC-6: prism-no-string-credentials does NOT fire on 'let api_key: String = get_secret()' — stub pattern not implemented (Red Gate)"
  fi
fi

echo "1..$TAP_COUNT"
exit $FAIL
