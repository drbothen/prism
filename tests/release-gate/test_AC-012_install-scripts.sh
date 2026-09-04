#!/usr/bin/env bash
# S-REL-003 AC-001..AC-009: Consumer install scripts structural assertions.
#
# Asserts that scripts/install.sh and scripts/install.ps1 exist with the
# required structure and that the CI and release workflows have the required
# job definitions and upload steps.
#
# Tests covered:
#   AC-001: install.sh exists and shellcheck-clean (structural check — CI job asserts shellcheck)
#   AC-002: platform detection covers all 5 targets (4 in install.sh, Windows in install.ps1)
#   AC-003: SHA-256 checksum verification present + abort on mismatch
#   AC-004: PATH guidance present
#   AC-005: install.ps1 exists with PS 5.1+ compliance markers
#   AC-006: install.ps1 aborts on checksum mismatch
#   AC-007: both scripts use /releases?per_page=1 (not /releases/latest)
#   AC-009: ci.yml has shellcheck-install-scripts + psscriptanalyzer-install-ps1 jobs
#   AC-010: release.yml publish-release job uploads install.sh + install.ps1
#
# NOTE: Actual shellcheck execution happens in the ci.yml shellcheck-install-scripts
# job; PSScriptAnalyzer runs in the psscriptanalyzer-install-ps1 job.
# These structural tests verify the file content and CI/release workflow wiring.
#
# Red Gate state: ALL ASSERTIONS EXPECTED TO FAIL before S-REL-003 implementation.
#   (scripts/install.sh and scripts/install.ps1 do not yet exist; ci.yml does not
#    yet have the two new jobs; release.yml does not yet upload install scripts)
#
# After implementation: all 26 assertions should pass.
#
# Stories: S-REL-003 | Wave: F-A | Cycle: v1.0.0-release-engineering
# Traces to: delta-analysis.md §2.1 + §8; research U8/U9/U10/U29/U30/U33; ADJ-002
#
# Compatible with bash 3.2+ (macOS default).

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/tap_lib.sh"

WORKTREE="$(cd "${SCRIPT_DIR}/../.." && pwd)"
INSTALL_SH="${WORKTREE}/scripts/install.sh"
INSTALL_PS1="${WORKTREE}/scripts/install.ps1"
CI_YML="${WORKTREE}/.github/workflows/ci.yml"
REL_YML="${WORKTREE}/.github/workflows/release.yml"

# ===========================================================================
# AC-001 / AC-002 / AC-003 / AC-004: install.sh structure
# ===========================================================================

# AC-001: file exists
assert_file_exists "$INSTALL_SH" "AC-001"

# AC-001: set -euo pipefail (shellcheck-discipline header)
assert_contains "$INSTALL_SH" "set -euo pipefail" "AC-001"

# AC-003: Checksum mismatch abort message present
assert_contains "$INSTALL_SH" "Checksum mismatch" "AC-003"

# AC-003: CHECKSUM_CMD dual-path detection (sha256sum)
assert_contains "$INSTALL_SH" "sha256sum" "AC-003"

# AC-003: CHECKSUM_CMD dual-path detection (shasum fallback, macOS)
assert_contains "$INSTALL_SH" "shasum" "AC-003"

# AC-002: aarch64-apple-darwin target present (macOS ARM)
assert_contains "$INSTALL_SH" "aarch64-apple-darwin" "AC-002"

# AC-002: x86_64-apple-darwin target present (macOS Intel)
assert_contains "$INSTALL_SH" "x86_64-apple-darwin" "AC-002"

# AC-002: x86_64-unknown-linux-gnu target present (Linux glibc)
assert_contains "$INSTALL_SH" "x86_64-unknown-linux-gnu" "AC-002"

# AC-002: x86_64-unknown-linux-musl target present (Linux musl)
assert_contains "$INSTALL_SH" "x86_64-unknown-linux-musl" "AC-002"

# AC-002: x86_64-pc-windows-msvc referenced (Windows users directed to install.ps1)
assert_contains "$INSTALL_SH" "x86_64-pc-windows-msvc" "AC-002"

# AC-002: musl composite detection — getconf GNU_LIBC_VERSION (U10 probe 1)
assert_contains "$INSTALL_SH" "GNU_LIBC_VERSION" "AC-002"

# AC-002: musl composite detection — ld-musl-x86_64.so.1 path (U10 probe 2)
assert_contains "$INSTALL_SH" "ld-musl-x86_64.so.1" "AC-002"

# AC-007: uses /releases?per_page=1 (includes prereleases — NOT /releases/latest)
assert_contains "$INSTALL_SH" "releases?per_page=1" "AC-007"

# AC-007: does NOT use /releases/latest in functional lines (U8: excludes prereleases)
assert_not_in_functional_lines "$INSTALL_SH" "/releases/latest" "AC-007"

# AC-004: PATH guidance present (export PATH or PATH=)
if grep -qE '(export PATH|PATH guidance|not in your PATH)' "$INSTALL_SH" 2>/dev/null; then
  tap_pass "AC-004: PATH guidance present in install.sh"
else
  tap_fail "AC-004: PATH guidance absent from install.sh" \
    "AC-004 FAIL: expected PATH guidance (export PATH or 'not in your PATH') — not found"
fi

# Injection safety: no ${{ }} Actions expression patterns in functional lines
# (would indicate accidental copy-paste from workflow files)
assert_not_in_functional_lines_re "$INSTALL_SH" '\$\{\{' "AC-001-injection-safety"

# ===========================================================================
# AC-005 / AC-006 / AC-007: install.ps1 structure
# ===========================================================================

# AC-005: file exists
assert_file_exists "$INSTALL_PS1" "AC-005"

# AC-005: #Requires -Version 5.1 at the top (U29)
assert_contains "$INSTALL_PS1" "#Requires -Version 5.1" "AC-005"

# AC-005: Get-FileHash for SHA-256 (built-in PS 5+, no external dep)
assert_contains "$INSTALL_PS1" "Get-FileHash" "AC-005"

# AC-005: targets x86_64-pc-windows-msvc
assert_contains "$INSTALL_PS1" "x86_64-pc-windows-msvc" "AC-005"

# AC-006: checksum mismatch abort present
assert_contains "$INSTALL_PS1" "Checksum mismatch" "AC-006"

# AC-007: uses releases?per_page=1 (includes prereleases — NOT /releases/latest)
assert_contains "$INSTALL_PS1" "releases?per_page=1" "AC-007"

# ===========================================================================
# AC-009: CI workflow has shellcheck-install-scripts and PSScriptAnalyzer jobs
# ===========================================================================

# shellcheck-install-scripts job must be defined
if grep -qE '^  shellcheck-install-scripts:' "$CI_YML" 2>/dev/null; then
  tap_pass "AC-009: shellcheck-install-scripts job defined in ci.yml"
else
  tap_fail "AC-009: shellcheck-install-scripts job missing from ci.yml" \
    "AC-009 FAIL: 'shellcheck-install-scripts' job not found in ci.yml (S-REL-003 AC-009)"
fi

# psscriptanalyzer-install-ps1 job must be defined
if grep -qE '^  psscriptanalyzer-install-ps1:' "$CI_YML" 2>/dev/null; then
  tap_pass "AC-009: psscriptanalyzer-install-ps1 job defined in ci.yml"
else
  tap_fail "AC-009: psscriptanalyzer-install-ps1 job missing from ci.yml" \
    "AC-009 FAIL: 'psscriptanalyzer-install-ps1' job not found in ci.yml (S-REL-003 AC-009)"
fi

# ===========================================================================
# AC-010: release.yml publish-release job uploads install scripts (ADJ-002)
# ===========================================================================

# install.sh must be referenced inside the publish-release job
PUBLISH_BLOCK="$(awk '/^  publish-release:/{f=1;next} /^  [a-z][a-zA-Z-]*:/{f=0} f' "$REL_YML")"

if echo "$PUBLISH_BLOCK" | grep -qF 'scripts/install.sh' 2>/dev/null; then
  tap_pass "AC-010: scripts/install.sh referenced in publish-release job of release.yml"
else
  tap_fail "AC-010: scripts/install.sh missing from publish-release job of release.yml" \
    "AC-010 FAIL: 'scripts/install.sh' not found in publish-release job block (ADJ-002)"
fi

# install.ps1 must be referenced inside the publish-release job
if echo "$PUBLISH_BLOCK" | grep -qF 'scripts/install.ps1' 2>/dev/null; then
  tap_pass "AC-010: scripts/install.ps1 referenced in publish-release job of release.yml"
else
  tap_fail "AC-010: scripts/install.ps1 missing from publish-release job of release.yml" \
    "AC-010 FAIL: 'scripts/install.ps1' not found in publish-release job block (ADJ-002)"
fi

tap_done
