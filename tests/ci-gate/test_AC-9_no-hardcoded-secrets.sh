#!/usr/bin/env bash
# AC-9: No secrets appear hardcoded in workflow files — all sensitive values use secrets.VARNAME.
# KNOWN-LIMITATION: We cannot verify GitHub Actions runtime masking (that requires live execution).
#   This test verifies structural compliance: secret names appear only as secrets.VARNAME references,
#   and no raw token-shaped strings appear in the YAML files.
# requires: bash 3.2+

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
WORKFLOWS_DIR="${WORKTREE}/.github/workflows"

# All three workflow files must exist.
for wf in ci.yml post-merge.yml release.yml; do
  assert_file_exists "${WORKFLOWS_DIR}/${wf}" "AC-9"
done

# Each secret that story requires must appear as secrets.VARNAME, not inline.
EXPECTED_SECRETS=(
  "secrets.HOMEBREW_TAP_TOKEN"
  "secrets.CHOCOLATEY_API_KEY"
  "secrets.CRATES_IO_TOKEN"
)

# Check that each required secret reference is used (not yet — stubs lack them entirely).
for secret_ref in "${EXPECTED_SECRETS[@]}"; do
  found=false
  for wf in ci.yml post-merge.yml release.yml; do
    if grep -qF "$secret_ref" "${WORKFLOWS_DIR}/${wf}" 2>/dev/null; then
      found=true
      break
    fi
  done
  if $found; then
    tap_pass "AC-9: '${secret_ref}' is referenced in a workflow file"
  else
    tap_fail "AC-9: '${secret_ref}' not found in any workflow file" \
      "AC-9 FAIL: expected '${secret_ref}' to appear in release.yml — secret reference missing entirely"
  fi
done

# Negative: no bare token-shaped strings (64-char hex or base64 blobs) in workflow files.
# We check for patterns that look like raw API keys: 32+ alphanum chars not inside '${{ ... }}'.
# This is a heuristic, not exhaustive.
for wf in ci.yml post-merge.yml release.yml; do
  file="${WORKFLOWS_DIR}/${wf}"
  if [ -f "$file" ]; then
    # Look for lines containing long alphanumeric strings not inside ${{ }} expressions.
    if grep -qEv '\$\{\{' "$file" 2>/dev/null && \
       grep -qE '^[^#]*[A-Za-z0-9]{40,}' "$file" 2>/dev/null; then
      tap_fail "AC-9: possible hardcoded secret in ${wf}" \
        "AC-9 FAIL: found long alphanumeric string outside \${{ }} in ${wf} — manual review required"
    else
      tap_pass "AC-9: no obvious hardcoded secret pattern in ${wf}"
    fi
  fi
done

tap_done
