# AC-005: Claroty legacy tests still pass (excluding harness_tests)
#
# Runs all prism-dtu-claroty tests EXCEPT harness_tests to confirm the 60
# legacy tests (AC-001 through AC-008, edge cases, fidelity, multi-tenant,
# configure, configure-deny-unknown, configure-requires-admin-token) still
# pass after the harness migration.
# Traces to: BC-3.5.001 — existing AC coverage preserved

Output AC-005-legacy-tests-still-pass.gif
Output AC-005-legacy-tests-still-pass.webm

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

Type "# AC-005: prism-dtu-claroty legacy tests — 60/60 still pass (S-3.4.01)"
Enter
Sleep 500ms

Type "cargo test -p prism-dtu-claroty --features dtu 2>&1"
Enter
Sleep 30s
Sleep 2s
