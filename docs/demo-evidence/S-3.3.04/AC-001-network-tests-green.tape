# AC-001: 16/16 network isolation tests GREEN (BC-3.5.002)
#
# Runs the full network_isolation_test suite for prism-dtu-harness (--features dtu).
# Demonstrates that all 16 BC-3.5.002 tests pass with 0 failures.

Output AC-001-network-tests-green.gif
Output AC-001-network-tests-green.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 120s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.04"
  Enter
  Sleep 500ms
Show

Type "# AC-001: All 16 network_isolation_test tests GREEN (BC-3.5.002)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test network_isolation_test -- --nocapture 2>&1 | tail -50"
Enter
Sleep 120s
