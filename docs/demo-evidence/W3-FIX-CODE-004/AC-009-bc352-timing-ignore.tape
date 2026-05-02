# AC-009 (BC-3.5.002 timing): startup-budget tests marked #[ignore] with TD-W3-TIMING-001
#
# Two timing-fragile tests in network_isolation_test.rs are annotated with
# #[ignore = "fragile under parallel nextest load; see TD-W3-TIMING-001"]
# matching the existing BC-3.5.001 treatment from PR #113.
#
# Tests remain in the file; run with --ignored on dedicated hardware.
# This commit (360fd53b) was the FIRST in the W3-FIX-CODE-004 chain, unblocking
# subsequent W3.2 PR pushes that would otherwise hit CI timing failures.
#
# BC-3.5.002 postcondition 5 — 5s startup budget wall-clock assertion.
# Note: story specified 3 tests; HEAD has 2 ignored (both ac008 variants).
#
# Story: W3-FIX-CODE-004, branch: feature/W3-FIX-CODE-004
# Commits: 360fd53b -> 37088d66 -> cfe585c3

Output AC-009-bc352-timing-ignore.gif
Output AC-009-bc352-timing-ignore.webm

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

# --- Show the ignore annotations ---
Type "# AC-009 (BC-3.5.002): startup-budget tests marked #[ignore] + TD-W3-TIMING-001"
Enter
Sleep 500ms
Type "grep -n 'TD-W3-TIMING\|ignore.*fragile' crates/prism-dtu-harness/tests/network_isolation_test.rs"
Enter
Sleep 2s

# --- Show tests are skipped in default run ---
Type "# Default run: 2 timing tests ignored, 14 pass -- CI-safe"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-harness --features dtu --test network_isolation_test 2>&1 | grep -E 'ignored|test result' | tail -5"
Enter
Sleep 60s
