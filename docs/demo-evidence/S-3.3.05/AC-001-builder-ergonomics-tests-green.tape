# AC-001: Builder ergonomics test suite — 19/19 GREEN
#
# Runs the full builder_ergonomics_test suite for prism-dtu-harness (--features dtu).
# Demonstrates that all 19 new tests introduced in S-3.3.05 pass.
# Traces to: BC-3.5.001, BC-3.5.002, BC-3.6.001

Output AC-001-builder-ergonomics-tests-green.gif
Output AC-001-builder-ergonomics-tests-green.webm

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
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.3.05"
  Enter
  Sleep 500ms
Show

Type "# AC-001: builder_ergonomics_test — 19/19 tests GREEN (S-3.3.05)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test builder_ergonomics_test -- --nocapture 2>&1 | tail -35"
Enter

Wait+Screen /test result/
Sleep 2s
