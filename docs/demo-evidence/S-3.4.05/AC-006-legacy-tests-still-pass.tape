# AC-006 — Slack/PagerDuty/Jira legacy tests still pass
# Traces to: BC-3.5.001 postcondition 1 — no regression in fidelity/ac tests
# Story: S-3.4.05
#
# Shows the original ac_tests (Slack), fidelity tests, and org_tagging tests
# all still passing — migration to harness did not break existing coverage.
# ac_tests: 14, fidelity: 1+17+28=46, org_tagging: 7+8+8=23 legacy tests.
#
# VHS tape — produces AC-006-legacy-tests-still-pass.{gif,webm}

Output docs/demo-evidence/S-3.4.05/AC-006-legacy-tests-still-pass.gif
Output docs/demo-evidence/S-3.4.05/AC-006-legacy-tests-still-pass.webm

Require bash
Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms
Set WaitTimeout 180s

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.05"
Enter
Sleep 1s
Show

# Demo — run all tests across three crates; show suite counts and pass results
Type "cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --features dtu 2>&1 | grep -E 'running [0-9]+ test|test result: ok'"
Enter

Wait+Screen /test result: ok/
Sleep 2s
