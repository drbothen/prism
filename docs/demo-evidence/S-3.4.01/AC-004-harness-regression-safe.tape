# AC-004: prism-dtu-harness regression safety — 70/70 still pass
#
# Runs the full prism-dtu-harness test suite to confirm all 70 prior tests
# continue to pass after the Claroty harness migration in S-3.4.01.
# Traces to: BC-3.5.001, BC-3.5.002 — harness invariants preserved

Output AC-004-harness-regression-safe.gif
Output AC-004-harness-regression-safe.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 700
Set FontSize 14
Set Padding 20
Set WaitTimeout 600s
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.01"
  Enter
  Sleep 500ms
Show

Type "# AC-004: prism-dtu-harness — 70/70 regression-safe (S-3.4.01)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu 2>&1"
Enter
Sleep 30s
Sleep 2s
