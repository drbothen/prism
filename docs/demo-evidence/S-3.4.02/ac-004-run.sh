#!/usr/bin/env bash
set -euo pipefail
cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.02
cargo test -p prism-dtu-harness --features dtu 2>&1 | grep "test result"
