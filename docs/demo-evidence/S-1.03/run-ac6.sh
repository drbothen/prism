#!/usr/bin/env bash
export PATH="$HOME/.cargo/bin:$PATH"
cd /Users/jmagady/dev/prism/.worktrees/S-1.03-capability-resolution
echo "S-1.03 AC-6: parent() traversal"
echo ""
cargo test -p prism-core test_S_1_03_ac6 2>&1 | grep -E "running [0-9]|test tests::capability|test result"
echo ""
echo "Error-path: parent of single-segment returns None"
cargo test -p prism-core test_S_1_03_ec_parent_of_single 2>&1 | grep -E "running [0-9]|test tests::capability|test result"
