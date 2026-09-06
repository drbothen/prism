#!/usr/bin/env bash
# check-version-agnostic-docs.sh
#
# CI-wiring deferred to follow-up S-REL-DOCS-CI-WIRE-001 (post-beta.1);
# run in the delivery flow for beta.1.
#
# Red Gate check for S-REL-DOCS-AGNOSTIC-001 (AC-001..AC-008) + F-VID-P1-MED-002.
#
# Verifies that:
#   AC-001: docs/SETUP.md contains no hardcoded v1.0.0-rc. strings
#   AC-002: docs/SETUP.md AND README.md do not contain /releases/latest/download/ URL
#            (story v1.1: grep '/releases/latest/download/' docs/SETUP.md README.md = 0 matches)
#   AC-003: docs/SETUP.md AND README.md each contain a reference to /releases path
#            (story v1.1: grep '/releases' docs/SETUP.md ≥1 AND grep '/releases' README.md ≥1)
#   AC-004: scripts/install.sh and scripts/install.ps1 contain no hardcoded v1.0.0-rc. strings
#   AC-005: RELEASING.md §1 documents the pre-release exception (ADR-064 D2)
#   AC-006: Full sweep — no hardcoded v1.0.0-rc. anywhere in docs/SETUP.md scripts/ RELEASING.md README.md
#   AC-006b: Full sweep — no non-v-prefixed pre-release semver (X.Y.Z-channel.N) in
#            docs/SETUP.md scripts/ RELEASING.md README.md (F-VID-P1-MED-002: catches "1.0.0-rc.2" etc.)
#   AC-007: README.md contains no hardcoded v1.0.0-rc. strings
#   AC-008: CI-wiring confirmed by S-REL-DOCS-CI-WIRE-001 (post-beta.1); passing run confirms docs are version-agnostic.
#
# Authority: ADR-064 D1/D2, S-REL-DOCS-AGNOSTIC-001 v1.1,
#            S-REL-VERSION-IDENTITY pass-6 OBS-1 (README.md extension),
#            S-REL-VERSION-IDENTITY pass-7 MED-1 (AC-002/AC-003 README.md gap).
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
# AC-007: README.md contains no hardcoded v1.0.0-rc. strings
# (OBS-1, S-REL-VERSION-IDENTITY pass-6 — same treatment as docs/SETUP.md)
# ---------------------------------------------------------------------------
RC1_README=$(grep -c 'v1\.0\.0-rc\.' README.md 2>/dev/null || true)
if [ "$RC1_README" -eq 0 ]; then
    pass "AC-007: README.md contains no v1.0.0-rc. strings"
else
    fail "AC-007: README.md contains ${RC1_README} hardcoded v1.0.0-rc. string(s). \
Remove them and replace with version-agnostic instructions per S-REL-VERSION-IDENTITY pass-6 OBS-1."
    echo "       Occurrences:"
    grep -n 'v1\.0\.0-rc\.' README.md | sed 's/^/         /' || true
fi

# ---------------------------------------------------------------------------
# AC-002: docs/SETUP.md AND README.md do not contain /releases/latest/download/ URL
# (GitHub /latest/ excludes pre-releases — operators need the Releases page)
# Story v1.1: grep '/releases/latest/download/' docs/SETUP.md README.md = 0 matches.
# MED-1 (S-REL-VERSION-IDENTITY pass-7): extended from SETUP.md-only to both files.
# ---------------------------------------------------------------------------
LATEST_DL_SETUP=$(grep -c '/releases/latest/download/' docs/SETUP.md 2>/dev/null || true)
LATEST_DL_README=$(grep -c '/releases/latest/download/' README.md 2>/dev/null || true)
LATEST_DL=$((LATEST_DL_SETUP + LATEST_DL_README))
if [ "$LATEST_DL" -eq 0 ]; then
    pass "AC-002: docs/SETUP.md and README.md do not contain /releases/latest/download/"
else
    fail "AC-002: /releases/latest/download/ found in swept files (${LATEST_DL} occurrence(s)). \
/latest/ excludes pre-releases. Replace with GitHub Releases page instructions per \
S-REL-DOCS-AGNOSTIC-001 AC-002."
    if [ "$LATEST_DL_SETUP" -gt 0 ]; then
        echo "       docs/SETUP.md occurrences:"
        grep -n '/releases/latest/download/' docs/SETUP.md | sed 's/^/         /' || true
    fi
    if [ "$LATEST_DL_README" -gt 0 ]; then
        echo "       README.md occurrences:"
        grep -n '/releases/latest/download/' README.md | sed 's/^/         /' || true
    fi
fi

# ---------------------------------------------------------------------------
# AC-003: docs/SETUP.md AND README.md each reference the GitHub Releases page (/releases)
# Story v1.1: grep '/releases' docs/SETUP.md ≥1 AND grep '/releases' README.md ≥1.
# Anchored to "/releases" path (not bare word) to prevent false-green on prose-only mentions.
# (OBS-2, S-REL-VERSION-IDENTITY pass-6; extended to README.md in pass-7 MED-1)
# Both files are checked independently — a missing reference in EITHER file is a failure.
# ---------------------------------------------------------------------------
RELEASES_SETUP=$(grep -ci '/releases' docs/SETUP.md 2>/dev/null || true)
RELEASES_README=$(grep -ci '/releases' README.md 2>/dev/null || true)
if [ "$RELEASES_SETUP" -gt 0 ] && [ "$RELEASES_README" -gt 0 ]; then
    pass "AC-003: docs/SETUP.md (${RELEASES_SETUP} match(es)) and README.md (${RELEASES_README} match(es)) \
both reference the GitHub Releases page (/releases)"
else
    if [ "$RELEASES_SETUP" -eq 0 ]; then
        fail "AC-003: docs/SETUP.md does not reference the GitHub Releases page path (/releases). Add a \
download section directing operators to the Releases page per S-REL-DOCS-AGNOSTIC-001 AC-003."
    fi
    if [ "$RELEASES_README" -eq 0 ]; then
        fail "AC-003: README.md does not reference the GitHub Releases page path (/releases). Add a \
download section directing operators to the Releases page per S-REL-DOCS-AGNOSTIC-001 AC-003."
    fi
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
# AC-006: Full sweep — no hardcoded v-prefixed pre-release semver in docs/SETUP.md, scripts/, RELEASING.md, README.md, prism.toml.example
# This is the definitive gate; it catches any occurrence not covered by AC-001/004/007.
# This script is excluded from the sweep because it legitimately names the pattern
# it checks for (test infrastructure, not install documentation).
# README.md added to sweep: OBS-1, S-REL-VERSION-IDENTITY pass-6.
# prism.toml.example added to sweep: B-9, S-REL-VERSION-IDENTITY review-cycle-4.
# Scope is docs/SETUP.md (not all of docs/) — docs/RELEASE-CHANNELS.md and other
# docs/ files legitimately use concrete pre-release semver examples as instructional
# content (tag format examples, maturity model illustrations). Narrowing to
# docs/SETUP.md ensures only operator-facing install documentation is gated.
#
# B-4 fix (S-REL-VERSION-IDENTITY review-cycle-2): generalized pattern from the specific
# 'v1.0.0-rc.' literal to 'v[X.Y.Z-(rc|beta|alpha|nightly).N]' so that version strings
# like 'v1.0.0-beta.1' or 'v1.0.0-alpha.2' are also caught. AC-001/AC-004/AC-007 retain
# their specific v1.0.0-rc. checks for targeted diagnostics; AC-006 is the general gate.
# ---------------------------------------------------------------------------
SELF="$(basename "${BASH_SOURCE[0]}")"
# Use || true to prevent set -o pipefail from aborting when grep finds zero matches
# (grep exits 1 with no output; wc -l would output "0" correctly, but pipefail
# propagates the grep non-zero exit through the pipeline assignment).
FULL_SWEEP=$(grep -rE 'v[0-9]+\.[0-9]+\.[0-9]+-(rc|beta|alpha|nightly)\.[0-9]+' \
    docs/SETUP.md scripts/ RELEASING.md README.md prism.toml.example \
    --exclude="$SELF" 2>/dev/null | wc -l | tr -d ' ') || FULL_SWEEP=0
if [ "$FULL_SWEEP" -eq 0 ]; then
    pass "AC-006: Full sweep — no hardcoded v-prefixed pre-release semver (v[X.Y.Z-(rc|beta|alpha|nightly).N]) \
found in docs/SETUP.md scripts/ RELEASING.md README.md prism.toml.example"
else
    fail "AC-006: Full sweep found ${FULL_SWEEP} hardcoded v-prefixed pre-release semver string(s) \
(e.g. v1.0.0-rc.1, v1.0.0-beta.1). Fix all occurrences before declaring S-REL-DOCS-AGNOSTIC-001 complete."
    echo "       Occurrences:"
    grep -rnE 'v[0-9]+\.[0-9]+\.[0-9]+-(rc|beta|alpha|nightly)\.[0-9]+' \
        docs/SETUP.md scripts/ RELEASING.md README.md prism.toml.example \
        --exclude="$SELF" | sed 's/^/         /' || true
fi

# ---------------------------------------------------------------------------
# AC-006b: Full sweep — no non-v-prefixed pre-release semver (F-VID-P1-MED-002)
# Catches strings like "1.0.0-rc.2" or "1.0.0-beta.1" (no leading v) that the
# AC-006 v1.0.0-rc. pattern misses. Pattern: X.Y.Z-channel.N where channel is
# rc, beta, alpha, or nightly. The develop default (1.0.0-dev) does not match (no .N suffix).
# This script is excluded because it legitimately names the pattern.
# Scope is docs/SETUP.md (not all of docs/) — same rationale as AC-006.
#
# MED-2 (S-REL-VERSION-IDENTITY pass-2): changed [^v] to (^|[^v]) so that a
# bare pre-release semver at column 0 (start of line) is also detected.
# The previous pattern [^v] required a non-v character BEFORE the digits, so
# "1.0.0-rc.2" at column 0 was missed. (^|[^v]) matches either start-of-line
# or a non-v character.  "v1.0.0-rc.2" still does NOT match because [^v]
# does not match 'v' and ^ does not apply mid-line.
# README.md added to sweep: OBS-1, S-REL-VERSION-IDENTITY pass-6.
# prism.toml.example added to sweep: B-9, S-REL-VERSION-IDENTITY review-cycle-4.
# ---------------------------------------------------------------------------
NONV_PRERELEASE=$(grep -rE '(^|[^v])[0-9]+\.[0-9]+\.[0-9]+-(rc|beta|alpha|nightly)\.[0-9]+' \
    docs/SETUP.md scripts/ RELEASING.md README.md prism.toml.example \
    --exclude="$SELF" 2>/dev/null | wc -l | tr -d ' ') || NONV_PRERELEASE=0
if [ "$NONV_PRERELEASE" -eq 0 ]; then
    pass "AC-006b: Full sweep — no non-v-prefixed pre-release semver (X.Y.Z-(rc|beta|alpha|nightly).N) found"
else
    fail "AC-006b: Full sweep found ${NONV_PRERELEASE} non-v-prefixed pre-release semver string(s) \
(e.g. '1.0.0-rc.2'). Replace with version-agnostic text per S-REL-DOCS-AGNOSTIC-001 \
(F-VID-P1-MED-002: use 'prism 1.0.0' or 'prism <version>' format examples)."
    echo "       Occurrences:"
    grep -rnE '(^|[^v])[0-9]+\.[0-9]+\.[0-9]+-(rc|beta|alpha|nightly)\.[0-9]+' \
        docs/SETUP.md scripts/ RELEASING.md README.md prism.toml.example \
        --exclude="$SELF" | sed 's/^/         /' || true
fi

# ---------------------------------------------------------------------------
# AC-009: prism.toml.example — no bare pre-release channel qualifiers in prose
# (B-9, S-REL-VERSION-IDENTITY review-cycle-4)
# Catches bare channel qualifiers like "rc.1", "beta.2", "alpha.3" that appear
# in TOML comment prose WITHOUT a leading vX.Y.Z- prefix. These slip past AC-006
# and AC-006b which both require the X.Y.Z component. A comment like "not part
# of the validated rc.1 surface" would not be caught by either general gate.
# Pattern: word boundary or non-alnum, then (rc|beta|alpha)\.\d+
# (nightly is excluded as it doesn't take a .N suffix in practice)
# This script is excluded (SELF) because it legitimately names these patterns.
# ---------------------------------------------------------------------------
BARE_CHANNEL=$(grep -E '(^|[^a-z])(rc|beta|alpha)\.[0-9]+' \
    prism.toml.example 2>/dev/null | wc -l | tr -d ' ') || BARE_CHANNEL=0
if [ "$BARE_CHANNEL" -eq 0 ]; then
    pass "AC-009: prism.toml.example — no bare pre-release channel qualifiers (rc.N, beta.N, alpha.N) in prose"
else
    fail "AC-009: prism.toml.example contains ${BARE_CHANNEL} bare pre-release channel qualifier(s) \
(e.g. 'rc.1', 'beta.2' without a vX.Y.Z- prefix). Remove version-specific qualifiers from \
comment prose (B-9, S-REL-VERSION-IDENTITY review-cycle-4)."
    echo "       Occurrences:"
    grep -nE '(^|[^a-z])(rc|beta|alpha)\.[0-9]+' prism.toml.example | sed 's/^/         /' || true
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "=== Results: ${PASS} passed, ${FAIL} failed ==="

if [ "$FAIL" -gt 0 ]; then
    echo ""
    echo "RED: ${FAIL} check(s) failed. Run again after editing"
    echo "README.md, docs/SETUP.md, scripts/install.sh, scripts/install.ps1, and RELEASING.md."
    exit 1
fi

echo ""
echo "GREEN: All checks pass. S-REL-DOCS-AGNOSTIC-001 AC-001, AC-002, AC-003, AC-004, AC-005, AC-006, AC-006b, AC-007, AC-009 satisfied."
exit 0
