# E2E: Aggregate run -- all 30 integration tests across all 13 ACs in one pass.
#
# Runs the full prism-dtu-demo-server test suite with features dtu,tls.
# Shows overall test result counts and per-AC file test results.

Output E2E-aggregate-all-acs.gif
Output E2E-aggregate-all-acs.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 750
Set FontSize 13
Set Padding 20
Set Shell "bash"

Hide
  Type "cd /Users/jmagady/Dev/prism/.worktrees/S-6.20-dtu-demo-server"
  Enter
  Sleep 500ms
Show

Type "# S-6.20 DTU Demo Server -- full suite (30 tests, 13 ACs, features dtu+tls)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-demo-server --features dtu,tls 2>&1 | grep -E 'test result|FAILED|ok\\.' | head -40"
Enter
Wait+Screen /test result.*ok/
Sleep 2s
