# AC-005 — Harness regression-safe (prism-dtu-harness 91-test suite clean)
# Traces to: BC-3.5.001 (VP-123, VP-124) — harness isolation invariants unchanged
# Story: S-3.4.05
#
# Demonstrates harness-based tests for all three crates (91 total) pass with no
# regressions — the harness library was not modified by this migration story.
# Slack: 24, PagerDuty: 28, Jira: 39 = 91 harness_tests green.
#
# VHS tape — produces AC-005-harness-regression-safe.{gif,webm}

Output docs/demo-evidence/S-3.4.05/AC-005-harness-regression-safe.gif
Output docs/demo-evidence/S-3.4.05/AC-005-harness-regression-safe.webm

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

# Demo — run all three harness_tests suites; show combined counts (91 total)
Type "cargo test -p prism-dtu-slack -p prism-dtu-pagerduty -p prism-dtu-jira --test harness_tests --features dtu 2>&1 | grep -E 'running [0-9]+ test|test result: ok'"
Enter

Wait+Screen /test result: ok/
Sleep 2s
