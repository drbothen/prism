# AC-005: Logical-mode regression safety — 34/34 still pass (BC-3.5.001)
#
# Runs the full logical_isolation_test suite after S-3.3.04 network-mode additions.
# Demonstrates that no regressions were introduced in logical isolation mode (34 tests pass).

Output AC-005-logical-mode-regression-safe.gif
Output AC-005-logical-mode-regression-safe.webm

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

Type "# AC-005: Logical-mode regression safety — 34/34 tests still GREEN after network-mode additions"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test 2>&1 | tail -45"
Enter
Sleep 120s
