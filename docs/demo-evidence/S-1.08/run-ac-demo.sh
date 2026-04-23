#!/usr/bin/env bash
# Helper script for VHS demos — runs cargo test with a specific filter
# and ensures output appears synchronously in the TTY
set -euo pipefail

WORKTREE="/Users/jmagady/dev/prism/.worktrees/S-1.08-feature-flags"
cd "$WORKTREE"

FILTER="${1:-}"
FEATURES="${2:---no-default-features}"

if [[ -n "$FILTER" ]]; then
  cargo test $FEATURES -p prism-security "$FILTER" 2>&1 | grep -E "^test |^test result"
else
  cargo test $FEATURES -p prism-security 2>&1 | grep -E "^test |^test result"
fi
