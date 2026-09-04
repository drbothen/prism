#!/usr/bin/env bash
# S-REL-003 AC-001..AC-010: Consumer install scripts structural assertions.
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
#   AC-008: install.sh optionally verifies build provenance via gh attestation verify
#   AC-009: ci.yml has shellcheck-install-scripts + psscriptanalyzer-install-ps1 jobs
#   AC-010: release.yml publish-release job uploads install.sh + install.ps1
#
# NOTE: Actual shellcheck execution happens in the ci.yml shellcheck-install-scripts
# job; PSScriptAnalyzer runs in the psscriptanalyzer-install-ps1 job.
# These structural tests verify the file content and CI/release workflow wiring.
#
# Red Gate state (PR fix cascade pass 1):
#   - 26 original assertions: PASS (implementation shipped)
#   - 3 N4/AC-008 assertions: PASS (provenance code exists)
#   - 2 N6-hardened AC-003 assertions: PASS (CHECKSUM_CMD block exists)
#   - 13 new red-gate assertions (SEC-001..006, B1, B2, B3, N2, N8): FAIL on HEAD
#     These assert the FIXED patterns and are RED until the implementer applies fixes.
#
# After all fixes applied: all 42 assertions should pass.
#
# Stories: S-REL-003 | Wave: F-A | Cycle: v1.0.0-release-engineering
# Traces to: delta-analysis.md §2.1 + §8; research U8/U9/U10/U29/U30/U33; ADJ-002
# PR fix cascade: SEC-001/002/003/004/005/006 + B1/B2/B3 + N2/N4/N5/N6/N8
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

# AC-003 (N6-hardened): CHECKSUM_CMD assignment for sha256sum — scoped to functional
# block assignment, not just comment mentions. Survives comment-only references.
assert_contains "$INSTALL_SH" "CHECKSUM_CMD=(sha256sum)" "AC-003"

# AC-003 (N6-hardened): CHECKSUM_CMD assignment for shasum fallback — scoped to
# functional block assignment. Detects deletion of the CHECKSUM_CMD detection block.
assert_contains "$INSTALL_SH" "CHECKSUM_CMD=(shasum" "AC-003"

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

# the 'shellcheck-install-scripts' job must be defined in ci.yml
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


# ===========================================================================
# AC-008: optional gh attestation provenance verification (N4: previously zero assertions)
# ===========================================================================

# AC-008: gh attestation verify must be present in install.sh
assert_contains "$INSTALL_SH" "gh attestation verify" "AC-008"

# AC-008: provenance check must be conditioned on gh CLI availability
assert_contains "$INSTALL_SH" "command -v gh" "AC-008"

# AC-008: --skip-verify-provenance flag must allow opt-out
assert_contains "$INSTALL_SH" "SKIP_VERIFY_PROVENANCE" "AC-008"

# ===========================================================================
# SEC-003: provenance verification failure must be BLOCKING (exit non-zero)
# RED: current HEAD WARNING+continue; fixed code must exit 1 in failure branch.
# ===========================================================================

# Extract the "Optional provenance verification" block (bounded by its section comments).
# If install.sh is restructured and those comments are absent, awk returns empty and
# the grep finds no exit 1, so tap_fail fires — that is the correct fail-closed result.
_PROV_BLOCK="$(awk '/Optional provenance verification/{f=1} /Extract and install/{f=0} f{print}' \
  "$INSTALL_SH" 2>/dev/null)"
if echo "$_PROV_BLOCK" | grep -qF 'exit 1' 2>/dev/null; then
  tap_pass "SEC-003: provenance failure path has blocking exit in install.sh"
else
  tap_fail "SEC-003: provenance failure is non-blocking (WARNING+continue) in install.sh" \
    "SEC-003 FAIL: gh attestation failure must exit 1 when gh present — currently WARNING-only"
fi

# ===========================================================================
# SEC-001 / SEC-002 / N8: PSScriptAnalyzer CI job hardening
# Extract the psscriptanalyzer-install-ps1 job block once and reuse for all three.
# ===========================================================================

_PSANALYZER_BLOCK="$(awk '/^  psscriptanalyzer-install-ps1:/{f=1;next} /^  [a-z][a-zA-Z-]*:/{f=0} f' \
  "$CI_YML" 2>/dev/null)"

# SEC-001: PSScriptAnalyzer step must be wired to FAIL the job when findings exist.
# Fixed code must capture results and exit 1 on non-empty findings (or use -EnableExit).
# RED: current HEAD has no exit 1 / -EnableExit / result-capture in the lint step.
if echo "$_PSANALYZER_BLOCK" | grep -qE '(exit 1|-EnableExit|\$results)' 2>/dev/null; then
  tap_pass "SEC-001: PSScriptAnalyzer lint step has failing-exit wiring in ci.yml"
else
  tap_fail "SEC-001: PSScriptAnalyzer lint step missing failing-exit wiring in ci.yml" \
    "SEC-001 FAIL: Invoke-ScriptAnalyzer must be wired to fail the job (exit 1 or -EnableExit)"
fi

# SEC-002: Install-Module PSScriptAnalyzer must carry an explicit version pin.
# RED: current HEAD uses Install-Module without -RequiredVersion (CWE-494).
if echo "$_PSANALYZER_BLOCK" | grep -qF 'RequiredVersion' 2>/dev/null; then
  tap_pass "SEC-002: Install-Module PSScriptAnalyzer has version pin (-RequiredVersion) in ci.yml"
else
  tap_fail "SEC-002: Install-Module PSScriptAnalyzer missing version pin (-RequiredVersion) in ci.yml" \
    "SEC-002 FAIL: Install-Module must use -RequiredVersion <version> to prevent supply-chain substitution"
fi

# N8: PSScriptAnalyzer must validate the PS 5.1 runtime target, not silently only
# run under pwsh/PS 7. Fixed code must have shell: powershell, CompatibilityTargeted,
# or a documented 5.1 scope marker.
# RED: current HEAD has only shell: pwsh (PS 7) with no PS 5.1 targeting.
if echo "$_PSANALYZER_BLOCK" | grep -qE '(shell: powershell|CompatibilityTargeted|5\.1)' 2>/dev/null; then
  tap_pass "N8: PSScriptAnalyzer job targets PS 5.1 runtime in ci.yml"
else
  tap_fail "N8: PSScriptAnalyzer job does not target PS 5.1 runtime" \
    "N8 FAIL: PSScriptAnalyzer must validate PS 5.1 compat (shell: powershell or CompatibilityTargeted)"
fi

# ===========================================================================
# SEC-004: checksum lookup must use fixed-string grep (not ARCHIVE-as-regex)
# RED: current HEAD uses plain grep without -F, treating ARCHIVE as a regex pattern.
# ===========================================================================

assert_contains "$INSTALL_SH" "grep -F" "SEC-004"

# ===========================================================================
# SEC-005: VERSION format must be validated before URL/path construction
# RED: neither script validates VERSION format on current HEAD.
# ===========================================================================

# install.sh: bash regex guard (=~ ^v<digits>)
if grep -qE '=~ \^v' "$INSTALL_SH" 2>/dev/null; then
  tap_pass "SEC-005: VERSION format validation (bash regex) present in install.sh"
else
  tap_fail "SEC-005: VERSION format validation absent from install.sh" \
    "SEC-005 FAIL: VERSION must be validated (e.g. [[ \${VERSION} =~ ^v[0-9] ]]) before URL construction"
fi

# install.ps1: PowerShell match/notmatch on \$Version
if grep -qE '\$Version -(not)?match' "$INSTALL_PS1" 2>/dev/null; then
  tap_pass "SEC-005: VERSION format validation present in install.ps1"
else
  tap_fail "SEC-005: VERSION format validation absent from install.ps1" \
    "SEC-005 FAIL: \$Version must be validated (e.g. -notmatch) before URL/path construction"
fi

# ===========================================================================
# SEC-006: Invoke-WebRequest calls must set a timeout
# RED: current HEAD has no -TimeoutSec on any Invoke-WebRequest call in install.ps1.
# ===========================================================================

assert_contains "$INSTALL_PS1" "-TimeoutSec" "SEC-006"

# ===========================================================================
# B1: installers must emit a post-install NOTICE about prism.toml.example + sensor specs
# Orchestrator-adjudicated: binary-only install is intentional; specs ship via demo bundle;
# installer must tell users where to find prism.toml.example and sensor spec files.
# RED: neither script mentions prism.toml on current HEAD.
# ===========================================================================

assert_contains "$INSTALL_SH" "prism.toml" "B1"
assert_contains "$INSTALL_PS1" "prism.toml" "B1"

# ===========================================================================
# B2: --version error guard must be reachable (not dead code under set -euo pipefail)
# Fixed code must use a $# bounds check or bash error-on-null (${2:?}) to ensure
# the guard fires before set -euo pipefail can kill the pipeline.
# RED: current HEAD uses ${2:-} with no prior $# check.
# ===========================================================================

if grep -qE '(\$# -lt 2|\$\{2:\?)' "$INSTALL_SH" 2>/dev/null; then
  tap_pass "B2: VERSION guard has reachable structure in install.sh"
else
  tap_fail "B2: VERSION guard may be unreachable dead code in install.sh" \
    "B2 FAIL: --version must use \$# -lt 2 or \${2:?} for reachable guard under set -euo pipefail"
fi

# ===========================================================================
# B3: install.ps1 PATH remediation must NOT duplicate machine PATH into user env
# Current code: SetEnvironmentVariable guidance uses $env:PATH (full process PATH,
# which includes machine PATH entries — duplicates them into user persistent scope).
# Fixed code: must reference $UserPath (user-scoped persistent PATH only).
# RED: current HEAD SetEnvironmentVariable guidance uses env:PATH.
# ===========================================================================

if grep -F 'SetEnvironmentVariable' "$INSTALL_PS1" 2>/dev/null | grep -qF 'env:PATH'; then
  tap_fail "B3: install.ps1 PATH guidance uses env:PATH (would duplicate machine PATH)" \
    "B3 FAIL: SetEnvironmentVariable must reference \$UserPath (user scope) not env:PATH (machine+user)"
else
  tap_pass "B3: install.ps1 PATH guidance uses user-scoped PATH (not env:PATH)"
fi

# ===========================================================================
# N2: install.ps1 must force TLS 1.2 for PS 5.1 compatibility
# PS 5.1 on older Windows defaults to TLS 1.0/1.1 for .NET WebClient/ServicePoint.
# Fixed code must set [Net.ServicePointManager]::SecurityProtocol before web calls.
# RED: current HEAD has no ServicePointManager configuration.
# ===========================================================================

assert_contains "$INSTALL_PS1" "ServicePointManager" "N2"

tap_done
