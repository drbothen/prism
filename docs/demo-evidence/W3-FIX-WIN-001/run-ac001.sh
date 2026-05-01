#!/usr/bin/env bash
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH"
cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-WIN-001
cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test test_BC_3_5_001_drop_releases_ports -- --nocapture
