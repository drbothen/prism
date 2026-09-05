#!/usr/bin/env bash
# check-version-agnostic-docs.sh
#
# Red Gate check for S-REL-DOCS-AGNOSTIC-001 (AC-001..AC-006).
#
# Verifies that:
#   AC-001: docs/SETUP.md contains no hardcoded v1.0.0-rc. strings
#   AC-002: docs/SETUP.md does not contain /releases/latest/download/ URL
#   AC-003: docs/SETUP.md references the GitHub Releases page for downloads
#   AC-004: scripts/install.sh and scripts/install.ps1 contain no hardcoded v1.0.0-rc. strings
#   AC-005: RELEASING.md §1 documents the pre-release exception (ADR-064 D2)
#   AC-006: Full sweep — no hardcoded v1.0.0-rc. anywhere in docs/ scripts/ RELEASING.md
#
# Authority: ADR-064 D1/D2, S-REL-DOCS-AGNOSTIC-001.
#
# Usage:
#   bash scripts/check-version-agnostic-docs.sh        # from workspace root
#   bash scripts/check-version-agnostic-docs.sh --ci   # same; exits 1 on first failure
#
# Exit codes:
#   0 = all checks pass (GREEN)
#   1 = one or more checks fail (RED)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PASS=0
FAIL=0

# Colour codes (only if terminal is interactive).
if [ -t 1 ]; then
    RED_C='\033[0;31m'
    GREEN_C='\033[0;32m'
    NC='\033[0m'
else
    RED_C=''
    GREEN_C=''
    NC=''
fi

pass() { echo -e "${GREEN_C}PASS${NC} $*"; PASS=$((PASS + 1)); }
fail() { echo -e "${RED_C}FAIL${NC} $*"; FAIL=$((FAIL + 1)); }

echo "=== S-REL-DOCS-AGNOSTIC-001: version-agnostic docs check ==="
echo "Working directory: $REPO_ROOT"
echo ""

# ---------------------------------------------------------------------------
# AC-001: docs/SETUP.md contains no hardcoded v1.0.0-rc. strings
# ---------------------------------------------------------------------------
RC1_SETUP=$(grep -c 'v1\.0\.0-rc\.' docs/SETUP.md 2>/dev/null || true)
if [ "$RC1_SETUP" -eq 0 ]; then
    pass "AC-001: docs/SETUP.md contains no v1.0.0-rc. strings"
else
    fail "AC-001: docs/SETUP.md contains ${RC1_SETUP} hardcoded v1.0.0-rc. string(s). \
Remove them and replace with version-agnostic instructions per S-REL-DOCS-AGNOSTIC-001."
    echo "       Occurrences:"
    grep -n 'v1\.0\.0-rc\.' docs/SETUP.md | sed 's/^/         /' || true
fi

# ---------------------------------------------------------------------------
# AC-002: docs/SETUP.md does not contain /releases/latest/download/ URL
# (GitHub /latest/ excludes pre-releases — operators need the Releases page)
# ---------------------------------------------------------------------------
LATEST_DL=$(grep -c '/releases/latest/download/' docs/SETUP.md 2>/dev/null || true)
if [ "$LATEST_DL" -eq 0 ]; then
    pass "AC-002: docs/SETUP.md does not contain /releases/latest/download/"
else
    fail "AC-002: docs/SETUP.md contains ${LATEST_DL} /releases/latest/download/ URL(s). \
/latest/ excludes pre-releases. Replace with GitHub Releases page instructions per \
S-REL-DOCS-AGNOSTIC-001 AC-002."
    echo "       Occurrences:"
    grep -n '/releases/latest/download/' docs/SETUP.md | sed 's/^/         /' || true
fi

# ---------------------------------------------------------------------------
# AC-003: docs/SETUP.md references the GitHub Releases page for downloads
# ---------------------------------------------------------------------------
RELEASES_PAGE=$(grep -ci 'releases' docs/SETUP.md 2>/dev/null || true)
if [ "$RELEASES_PAGE" -gt 0 ]; then
    pass "AC-003: docs/SETUP.md references GitHub Releases page (${RELEASES_PAGE} match(es))"
else
    fail "AC-003: docs/SETUP.md does not reference the GitHub Releases page. Add a \
download section directing operators to the Releases page per S-REL-DOCS-AGNOSTIC-001 AC-003."
fi

# ---------------------------------------------------------------------------
# AC-004: scripts/install.sh and scripts/install.ps1 contain no v1.0.0-rc. strings
# ---------------------------------------------------------------------------
RC1_INSTALL_SH=0
if [ -f scripts/install.sh ]; then
    RC1_INSTALL_SH=$(grep -c 'v1\.0\.0-rc\.' scripts/install.sh 2>/dev/null || true)
fi

RC1_INSTALL_PS1=0
if [ -f scripts/install.ps1 ]; then
    RC1_INSTALL_PS1=$(grep -c 'v1\.0\.0-rc\.' scripts/install.ps1 2>/dev/null || true)
fi

if [ "$RC1_INSTALL_SH" -eq 0 ] && [ "$RC1_INSTALL_PS1" -eq 0 ]; then
    pass "AC-004: scripts/install.sh and scripts/install.ps1 contain no v1.0.0-rc. strings"
else
    if [ "$RC1_INSTALL_SH" -gt 0 ]; then
        fail "AC-004: scripts/install.sh contains ${RC1_INSTALL_SH} hardcoded v1.0.0-rc. string(s)."
        echo "       Occurrences:"
        grep -n 'v1\.0\.0-rc\.' scripts/install.sh | sed 's/^/         /' || true
    fi
    if [ "$RC1_INSTALL_PS1" -gt 0 ]; then
        fail "AC-004: scripts/install.ps1 contains ${RC1_INSTALL_PS1} hardcoded v1.0.0-rc. string(s)."
        echo "       Occurrences:"
        grep -n 'v1\.0\.0-rc\.' scripts/install.ps1 | sed 's/^/         /' || true
    fi
fi

# ---------------------------------------------------------------------------
# AC-005: RELEASING.md §1 documents the pre-release exception (ADR-064 D2)
# The exception paragraph must state that develop carries 1.0.0-dev and
# that no Cargo.toml bump is needed between pre-releases.
# ---------------------------------------------------------------------------
if grep -q 'Pre-release exception' RELEASING.md 2>/dev/null; then
    # Verify the key claims are present in the exception paragraph.
    if grep -A10 'Pre-release exception' RELEASING.md | grep -q '1\.0\.0-dev'; then
        pass "AC-005: RELEASING.md §1 contains pre-release exception with 1.0.0-dev reference"
    else
        fail "AC-005: RELEASING.md contains 'Pre-release exception' heading but the \
1.0.0-dev version reference is missing. Add the full ADR-064 D2 paragraph per \
S-REL-DOCS-AGNOSTIC-001 AC-005."
    fi
else
    fail "AC-005: RELEASING.md §1 does not document the pre-release exception (ADR-064 D2). \
Add a paragraph explaining that develop carries 1.0.0-dev and no Cargo.toml bump is \
needed between pre-releases. See S-REL-DOCS-AGNOSTIC-001 AC-005 for the required text."
fi

# ---------------------------------------------------------------------------
# AC-006: Full sweep — no hardcoded v1.0.0-rc. in docs/, scripts/, or RELEASING.md
# This is the definitive gate; it catches any occurrence not covered by AC-001/004.
# This script is excluded from the sweep because it legitimately names the pattern
# it checks for (test infrastructure, not install documentation).
# ---------------------------------------------------------------------------
SELF="$(basename "${BASH_SOURCE[0]}")"
FULL_SWEEP=$(grep -r 'v1\.0\.0-rc\.' docs/ scripts/ RELEASING.md \
    --exclude="$SELF" 2>/dev/null | wc -l | tr -d ' ')
if [ "$FULL_SWEEP" -eq 0 ]; then
    pass "AC-006: Full sweep — no hardcoded v1.0.0-rc. strings found in docs/ scripts/ RELEASING.md"
else
    fail "AC-006: Full sweep found ${FULL_SWEEP} hardcoded v1.0.0-rc. string(s). \
Fix all occurrences before declaring S-REL-DOCS-AGNOSTIC-001 complete."
    echo "       Occurrences:"
    grep -rn 'v1\.0\.0-rc\.' docs/ scripts/ RELEASING.md \
        --exclude="$SELF" | sed 's/^/         /' || true
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "=== Results: ${PASS} passed, ${FAIL} failed ==="

if [ "$FAIL" -gt 0 ]; then
    echo ""
    echo "RED: ${FAIL} check(s) failed. This is the expected state before"
    echo "S-REL-DOCS-AGNOSTIC-001 implementation. Run again after editing"
    echo "docs/SETUP.md, scripts/install.sh, scripts/install.ps1, and RELEASING.md."
    exit 1
fi

echo ""
echo "GREEN: All checks pass. S-REL-DOCS-AGNOSTIC-001 AC-001..AC-006 satisfied."
exit 0
