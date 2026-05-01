#!/usr/bin/env bash
set -euo pipefail
cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.02
cargo test -p prism-dtu-armis --features dtu --test harness_tests 2>&1 | tail -4
