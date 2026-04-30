# AC-003: BC-3.6.001 — Failure injection scoped to one org
#
# Runs the BC-3.6.001 sub-suite of logical_isolation_test with --nocapture.
# Demonstrates that injected failures (timeout, auth-reject, rate-limit, malformed)
# are isolated to the target org and do not affect others.

Output AC-003-failure-injection.gif
Output AC-003-failure-injection.webm

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

Type "# AC-003: BC-3.6.001 — failure injection scoped to single org"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test logical_isolation_test test_BC_3_6_001 -- --nocapture 2>&1 | head -40"
Enter
Sleep 20s
