#!/usr/bin/env bash
set -euo pipefail
cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.02
cargo test -p prism-dtu-armis --features dtu 2>&1 | grep "test result" | grep -v " 0 passed"
