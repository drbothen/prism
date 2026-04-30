# AC-003: with_failure shorthand injects failure mode at build time
#
# Runs the with_failure tests: after .with_failure(slug, dtu_type, FailureMode::AuthReject),
# the first HTTP request to that clone returns 401 — no separate inject_failure call needed.
# Also covers FailureMode::None as a no-op and failure scoped to target org only.
# Traces to: BC-3.6.001 postcondition 1, S-3.3.05 Task 3

Output AC-003-with-failure-injection.gif
Output AC-003-with-failure-injection.webm

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

Type "# AC-003: with_failure pre-injects failure mode, observable on first request"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-harness --features dtu --test builder_ergonomics_test with_failure -- --nocapture 2>&1 | tail -35"
Enter

Wait+Screen /test result/
Sleep 2s
