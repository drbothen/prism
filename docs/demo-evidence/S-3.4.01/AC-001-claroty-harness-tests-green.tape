# AC-001: Claroty harness migration — 56/56 harness_tests GREEN
#
# Runs the full harness_tests suite for prism-dtu-claroty (--features dtu).
# Demonstrates that all 56 migrated tests pass via the harness-hosted clone.
# Traces to: BC-3.5.001, BC-3.5.002

Output AC-001-claroty-harness-tests-green.gif
Output AC-001-claroty-harness-tests-green.webm

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

Type "# AC-001: prism-dtu-claroty harness_tests — 56/56 GREEN (S-3.4.01)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-claroty --features dtu --test harness_tests -- --nocapture 2>&1"
Enter
Sleep 30s
Sleep 2s
