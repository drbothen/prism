#!/usr/bin/env bash
export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH"
cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-WIN-001
cargo test -p prism-dtu-harness --features dtu
