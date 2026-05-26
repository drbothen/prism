#!/usr/bin/env bash
# check-error-taxonomy-snapshot.sh
#
# PRR-008 fix (S-CONFIG-MULTI-TENANT-OVERRIDE-001 PR #155 fix-burst):
# Verify that `crates/prism-spec-engine/fixtures/error-taxonomy-snapshot.md`
# is byte-equal to the canonical E-SPEC-019..023 rows in the factory-artifacts
# error taxonomy source.
#
# Behavior:
#   - If the factory-artifacts worktree is mounted at .factory/ (local dev): runs
#     the comparison and exits non-zero on drift.
#   - If .factory/ is absent (CI default, orphan branch not checked out): exits 0
#     with a warning — the AC-005 include_str!() test covers code-vs-fixture drift
#     in all CI environments; this script only covers canonical-source vs fixture drift.
#
# Usage:
#   bash scripts/check-error-taxonomy-snapshot.sh
#   (Also runnable as a just recipe: `just check-taxonomy-snapshot`)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SNAPSHOT="$REPO_ROOT/crates/prism-spec-engine/fixtures/error-taxonomy-snapshot.md"
CANONICAL_TAXONOMY="$REPO_ROOT/.factory/specs/prd-supplements/error-taxonomy.md"

if [ ! -f "$CANONICAL_TAXONOMY" ]; then
    echo "[check-error-taxonomy-snapshot] SKIP: .factory/specs/prd-supplements/error-taxonomy.md not present"
    echo "  (factory-artifacts worktree not mounted — normal in CI)"
    echo "  The AC-005 include_str!() test covers code-vs-snapshot drift in CI."
    exit 0
fi

if [ ! -f "$SNAPSHOT" ]; then
    echo "[check-error-taxonomy-snapshot] FAIL: snapshot file not found at $SNAPSHOT"
    exit 1
fi

# Extract E-SPEC-019..023 rows from the canonical taxonomy.
# These are the rows relevant to the AC-005 test.
CANONICAL_ROWS=$(grep -E 'E-SPEC-0(19|20|21|22|23)' "$CANONICAL_TAXONOMY" || true)
SNAPSHOT_ROWS=$(grep -E 'E-SPEC-0(19|20|21|22|23)' "$SNAPSHOT" || true)

if [ "$CANONICAL_ROWS" = "$SNAPSHOT_ROWS" ]; then
    echo "[check-error-taxonomy-snapshot] PASS: snapshot matches canonical E-SPEC-019..023 rows"
    exit 0
else
    echo "[check-error-taxonomy-snapshot] FAIL: snapshot has drifted from canonical taxonomy"
    echo ""
    echo "=== Canonical rows (from .factory/) ==="
    echo "$CANONICAL_ROWS"
    echo ""
    echo "=== Snapshot rows (from fixture) ==="
    echo "$SNAPSHOT_ROWS"
    echo ""
    echo "Fix: update crates/prism-spec-engine/fixtures/error-taxonomy-snapshot.md to match"
    echo "     .factory/specs/prd-supplements/error-taxonomy.md E-SPEC-019..023 rows."
    echo "     Both files must be updated in the same commit."
    exit 1
fi
