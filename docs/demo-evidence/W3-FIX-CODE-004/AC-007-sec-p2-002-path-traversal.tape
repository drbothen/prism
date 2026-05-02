# AC-007 (SEC-P2-002): Pre-join path traversal checks fire before resolved.exists()
#
# Demonstrates that a traversal attempt targeting a non-existent path emits
# E-CFG-018 (SpecPathTraversal) rather than E-CFG-015 (SpecFileNotFound).
# The I/O-free checks (..) and absolute paths are now OUTSIDE the exists() gate.
#
# BC-3.3.004 CWE-22 invariant — pre-join checks must fire before existence test.
#
# Story: W3-FIX-CODE-004, branch: feature/W3-FIX-CODE-004
# Commits: 360fd53b -> 37088d66 -> cfe585c3

Output AC-007-sec-p2-002-path-traversal.gif
Output AC-007-sec-p2-002-path-traversal.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set WaitTimeout 120s

# --- Setup (hidden) ---
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-CODE-004"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

# --- Show the pre-join ordering in source ---
Type "# AC-007 (SEC-P2-002): pre-join .. check fires BEFORE resolved.exists()"
Enter
Sleep 500ms
Type "grep -n 'Step 1\|Step 2\|resolved.exists' crates/prism-customer-config/src/validator.rs"
Enter
Sleep 2s

# --- Run the traversal test ---
Type "# Test: non-existent traversal target -> E-CFG-018 (not E-CFG-015)"
Enter
Sleep 500ms
Type "cargo test -p prism-customer-config --test path_traversal -- test_BC_3_3_004_SEC_P2_002 2>&1 | tail -6"
Enter
Sleep 30s
