#!/usr/bin/env bash
# test_BC-3.7.001_AC-008_script-is-readonly.sh
#
# AC-008 / VP-136 / BC-3.7.001 invariant 2: running check-crate-layout.sh
# must not create, modify, or delete any file in the workspace.
#
# Implementation: compare git status --porcelain before and after.
#
# MUST FAIL at Red Gate: stub exits 1, failing the exit-code assertion.

set -uo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=tap_lib.sh
source "${SCRIPT_DIR}/tap_lib.sh"

SCRIPT="${WORKTREE}/scripts/check-crate-layout.sh"

# ── Test 1: script exits 0 (prerequisite for read-only to be meaningful) ─────
output=$(bash "${SCRIPT}" 2>&1)
exit_code=$?

if [ "${exit_code}" -eq 0 ]; then
  tap_pass "AC-008/VP-136: script exits 0 (conformant workspace — precondition for read-only check)"
else
  tap_fail "AC-008/VP-136: script exits 0 (conformant workspace — precondition for read-only check)" \
    "Exit code: ${exit_code}. Stub is active; real implementation required."
fi

# ── Test 2: git status unchanged after script run ────────────────────────────
if ! command -v git >/dev/null 2>&1; then
  tap_skip "AC-008/VP-136: git-status read-only check" "git not available"
else
  status_before=$(git -C "${WORKTREE}" status --porcelain 2>/dev/null)
  # Run script a second time (ignore exit code — only care about FS side effects).
  bash "${SCRIPT}" >/dev/null 2>&1 || true
  status_after=$(git -C "${WORKTREE}" status --porcelain 2>/dev/null)

  if [ "${status_before}" = "${status_after}" ]; then
    tap_pass "AC-008/VP-136: git status unchanged after script run (no file mutations)"
  else
    tap_fail "AC-008/VP-136: git status unchanged after script run (no file mutations)" \
      "Before: ${status_before} | After: ${status_after}"
  fi
fi

# ── Test 3: no temp files left behind by the script ──────────────────────────
# Look for common temp patterns (tmp files, .bak, lock files) introduced by script.
tmpfiles_before=$(find "${WORKTREE}" -maxdepth 3 -name "*.tmp" -o -name "*.bak" -o -name "*.lock.sh" 2>/dev/null | sort)
bash "${SCRIPT}" >/dev/null 2>&1 || true
tmpfiles_after=$(find "${WORKTREE}" -maxdepth 3 -name "*.tmp" -o -name "*.bak" -o -name "*.lock.sh" 2>/dev/null | sort)

if [ "${tmpfiles_before}" = "${tmpfiles_after}" ]; then
  tap_pass "AC-008: no .tmp/.bak files created by script"
else
  tap_fail "AC-008: no .tmp/.bak files created by script" \
    "New temp files detected: $(diff <(echo "${tmpfiles_before}") <(echo "${tmpfiles_after}") | grep '^>')"
fi

tap_done
