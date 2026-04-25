#!/usr/bin/env bash
# verify-sha-currency.sh — CHECKLIST #8 SHA Currency Check
#
# Encapsulates STATE-MANAGER-CHECKLIST.md command #8.
# Run before every state-manager burst push to factory-artifacts branch.
#
# Usage: bash .factory/hooks/verify-sha-currency.sh [--project-root PATH]
#
# Future: wire as pre-push hook for factory-artifacts branch when v0.52
# vsdd-factory plugin lands (wave-gate-prerequisite hook slot).
#
# Created: 2026-04-24 (Pass 3 remediation — OBS-002)

set -euo pipefail

# Resolve project root (default: parent of this script's containing dir)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FACTORY_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$FACTORY_DIR")"

# Allow override
while [[ $# -gt 0 ]]; do
  case "$1" in
    --project-root)
      PROJECT_ROOT="$2"
      FACTORY_DIR="$PROJECT_ROOT/.factory"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

STATE_MD="$FACTORY_DIR/STATE.md"
HANDOFF_MD="$FACTORY_DIR/SESSION-HANDOFF.md"

echo "=== SHA Currency Check (CHECKLIST #8) ==="
echo "Project root : $PROJECT_ROOT"
echo "factory dir  : $FACTORY_DIR"
echo ""

# --- Actual SHAs ---
ACTUAL_DEV=$(git -C "$PROJECT_ROOT" rev-parse develop 2>/dev/null || echo "ERR_NO_DEVELOP")
ACTUAL_FA=$(git -C "$FACTORY_DIR" rev-parse HEAD 2>/dev/null || echo "ERR_NO_FA")

ACTUAL_DEV_SHORT="${ACTUAL_DEV:0:8}"
ACTUAL_FA_SHORT="${ACTUAL_FA:0:8}"

echo "Actual develop HEAD      : $ACTUAL_DEV_SHORT"
echo "Actual factory-artifacts : $ACTUAL_FA_SHORT"
echo ""

# --- Cited SHAs in STATE.md ---
CITED_DEV_STATE=$(grep -oE 'develop_head: "?[0-9a-f]{8,40}' "$STATE_MD" 2>/dev/null \
  | head -1 | grep -oE '[0-9a-f]{8,40}' | cut -c1-8 || echo "NOT_FOUND")
CITED_FA_STATE=$(grep -oE 'factory-artifacts HEAD[^0-9a-f]*[0-9a-f]{8}' "$STATE_MD" 2>/dev/null \
  | head -1 | grep -oE '[0-9a-f]{8}' | tail -1 || echo "NOT_FOUND")

# --- Cited SHAs in SESSION-HANDOFF.md ---
CITED_DEV_HANDOFF=$(grep -oE 'develop HEAD[^|`]*`?[0-9a-f]{8}' "$HANDOFF_MD" 2>/dev/null \
  | head -1 | grep -oE '[0-9a-f]{8}' | tail -1 || echo "NOT_FOUND")
CITED_FA_HANDOFF=$(grep -oE 'factory-artifacts HEAD[^0-9a-f]*[0-9a-f]{8}' "$HANDOFF_MD" 2>/dev/null \
  | head -1 | grep -oE '[0-9a-f]{8}' | tail -1 || echo "NOT_FOUND")

echo "STATE.md    develop cited      : $CITED_DEV_STATE"
echo "STATE.md    factory-arts cited : $CITED_FA_STATE"
echo "HANDOFF.md  develop cited      : $CITED_DEV_HANDOFF"
echo "HANDOFF.md  factory-arts cited : $CITED_FA_HANDOFF"
echo ""

FAIL=0

# Check develop SHA — must match exactly
if [ "$ACTUAL_DEV_SHORT" != "$CITED_DEV_STATE" ]; then
  echo "FAIL: develop SHA in STATE.md is stale (cited=$CITED_DEV_STATE actual=$ACTUAL_DEV_SHORT)"
  FAIL=1
fi
if [ "$ACTUAL_DEV_SHORT" != "$CITED_DEV_HANDOFF" ]; then
  echo "FAIL: develop SHA in SESSION-HANDOFF.md is stale (cited=$CITED_DEV_HANDOFF actual=$ACTUAL_DEV_SHORT)"
  FAIL=1
fi

# Check factory-artifacts SHA — allow 1-commit two-commit-protocol drift ONLY when:
#   (a) HEAD's commit message contains "backfill" (Stage 2 is in-flight), AND
#   (b) HEAD^'s commit message does NOT contain "backfill" (strictly 2-commit chain).
# If HEAD^ also contains "backfill", this is a multi-commit chain extension — FAIL.
# Without both guards the exception masks 3+ commit chain bursts (OBS-001, Pass 4/5).
#
# Single canonical SHA discipline (Pass 5): a burst MUST use exactly 2 commits:
#   Commit 1 (Stage 1): all fixes + 15fa97e6 placeholder throughout all documents
#   Commit 2 (Stage 2): global replacement of 15fa97e6 with Commit 1's actual SHA
# NO third commit. If a fix is needed post-commit-2, reset --soft HEAD~2 and redo.
HEAD_MSG=$(git -C "$FACTORY_DIR" log -1 --format=%s 2>/dev/null || echo "")
PARENT_MSG=$(git -C "$FACTORY_DIR" log -1 --format=%s HEAD^ 2>/dev/null || echo "")
HEAD_IS_BACKFILL=0
PARENT_IS_BACKFILL=0
echo "$HEAD_MSG" | grep -qi "backfill" && HEAD_IS_BACKFILL=1
echo "$PARENT_MSG" | grep -qi "backfill" && PARENT_IS_BACKFILL=1

# Fabrication check: verify cited SHAs actually exist as git objects
if [ "$CITED_FA_STATE" != "NOT_FOUND" ]; then
  if ! git -C "$FACTORY_DIR" cat-file -e "${CITED_FA_STATE}"^{commit} 2>/dev/null; then
    echo "WARN: STATE.md cited factory-artifacts SHA $CITED_FA_STATE does not exist as a git object (FABRICATED?)"
  fi
fi
if [ "$CITED_FA_HANDOFF" != "NOT_FOUND" ]; then
  if ! git -C "$FACTORY_DIR" cat-file -e "${CITED_FA_HANDOFF}"^{commit} 2>/dev/null; then
    echo "WARN: SESSION-HANDOFF.md cited factory-artifacts SHA $CITED_FA_HANDOFF does not exist as a git object (FABRICATED?)"
  fi
fi

# Multi-commit chain detection: if HEAD^ also contains "backfill", the chain has 3+ commits
if [ "$HEAD_IS_BACKFILL" -eq 1 ] && [ "$PARENT_IS_BACKFILL" -eq 1 ]; then
  echo "FAIL: MULTI_COMMIT_CHAIN_NOT_ALLOWED — HEAD and HEAD^ both contain 'backfill' in commit message; the two-commit protocol permits exactly 1 fix commit + 1 backfill commit; chain extension detected (Pass 5 defect class)"
  FAIL=1
fi

if [ "$ACTUAL_FA_SHORT" != "$CITED_FA_STATE" ]; then
  PARENT_FA=$(git -C "$FACTORY_DIR" rev-parse HEAD^ 2>/dev/null | cut -c1-8 || echo "NO_PARENT")
  if [ "$PARENT_FA" = "$CITED_FA_STATE" ] && [ "$HEAD_IS_BACKFILL" -eq 1 ] && [ "$PARENT_IS_BACKFILL" -eq 0 ]; then
    echo "NOTE: factory-artifacts STATE.md cites HEAD^ ($CITED_FA_STATE) — within two-commit protocol exception (HEAD is backfill; HEAD^ is not)"
  else
    echo "FAIL: factory-artifacts SHA in STATE.md is stale (cited=$CITED_FA_STATE actual=$ACTUAL_FA_SHORT parent=$PARENT_FA head_is_backfill=$HEAD_IS_BACKFILL parent_is_backfill=$PARENT_IS_BACKFILL)"
    FAIL=1
  fi
fi
if [ "$ACTUAL_FA_SHORT" != "$CITED_FA_HANDOFF" ]; then
  PARENT_FA=$(git -C "$FACTORY_DIR" rev-parse HEAD^ 2>/dev/null | cut -c1-8 || echo "NO_PARENT")
  if [ "$PARENT_FA" = "$CITED_FA_HANDOFF" ] && [ "$HEAD_IS_BACKFILL" -eq 1 ] && [ "$PARENT_IS_BACKFILL" -eq 0 ]; then
    echo "NOTE: factory-artifacts HANDOFF.md cites HEAD^ ($CITED_FA_HANDOFF) — within two-commit protocol exception (HEAD is backfill; HEAD^ is not)"
  else
    echo "FAIL: factory-artifacts SHA in SESSION-HANDOFF.md is stale (cited=$CITED_FA_HANDOFF actual=$ACTUAL_FA_SHORT parent=$PARENT_FA head_is_backfill=$HEAD_IS_BACKFILL parent_is_backfill=$PARENT_IS_BACKFILL)"
    FAIL=1
  fi
fi

echo ""
if [ "$FAIL" -eq 0 ]; then
  echo "PASS: all SHA currency checks pass"
  exit 0
else
  echo "FAIL: SHA drift detected — fix cited SHAs before pushing"
  exit 1
fi
