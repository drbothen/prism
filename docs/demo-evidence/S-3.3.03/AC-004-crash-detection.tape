# AC-004: BC-3.6.002 — Crash detection within 1s; non-crashed clone unaffected
#
# Runs the BC-3.6.002 sub-suite of logical_isolation_test with --nocapture.
# Demonstrates: panic detected <1s, cause string preserved, non-crashed clone
# continues operating, and two simultaneous crashes are independent.

Output AC-004-crash-detection.gif
Output AC-004-crash-detection.webm

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

Type "# AC-004: BC-3.6.002 — crash detection within 1s, isolation from peers"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test test_BC_3_6_002 -- --nocapture 2>&1 | head -40"
Enter
Sleep 20s
