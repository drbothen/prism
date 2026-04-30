# AC-002: BC-3.5.001 — Logical isolation per-org
#
# Runs the BC-3.5.001 sub-suite of logical_isolation_test with --nocapture.
# Demonstrates per-org data segregation: Acme/Initech/Globex rows never cross-leak.

Output AC-002-logical-isolation.gif
Output AC-002-logical-isolation.webm

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

Type "# AC-002: BC-3.5.001 — logical isolation, per-org segregation"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test test_BC_3_5_001 -- --nocapture 2>&1 | head -40"
Enter
Sleep 20s
