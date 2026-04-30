# AC-001: All 34 harness tests GREEN
#
# Runs the full logical_isolation_test suite for prism-dtu-harness (--features dtu).
# Demonstrates that all 34 tests across BC-3.5.001, BC-3.6.001, BC-3.6.002 pass.

Output AC-001-all-34-tests-green.gif
Output AC-001-all-34-tests-green.webm

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
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.03"
  Enter
  Sleep 500ms
Show

Type "# AC-001: All 34 prism-dtu-harness logical isolation tests GREEN"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test 2>&1 | tail -45"
Enter
Sleep 20s
