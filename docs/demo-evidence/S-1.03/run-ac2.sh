#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:$PATH"
cd /Users/jmagady/dev/prism/.worktrees/S-1.03-capability-resolution
echo "S-1.03 AC-2: Parent Allow covers child path"
echo ""
cargo test -p prism-core test_S_1_03_ac2 2>&1 | grep -E "running [0-9]|test tests::capability|test result"
