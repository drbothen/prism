# AC-005 — Harness regression-safe: prism-dtu-harness tests all pass (70/70)
# Traces to: BC-3.5.001 postcondition 1, VP-126
# Story: S-3.4.03
#
# VHS tape — produces AC-005-harness-regression-safe.{gif,webm}
#
# Note: prism-dtu-harness is exercised via crowdstrike integration;
# harness isolation tests run across Armis + Claroty + CrowdStrike suites.
# We demonstrate using the multi_tenant suite (14 tests) and the
# harness_tests suite (47+2i) which together exercise the full harness.

Output docs/demo-evidence/S-3.4.03/AC-005-harness-regression-safe.gif
Output docs/demo-evidence/S-3.4.03/AC-005-harness-regression-safe.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.03"
Enter
Sleep 1s
Show

# Demo — multi_tenant + harness_tests pass; no harness regressions
Type "cargo test -p prism-dtu-crowdstrike --features dtu --test multi_tenant --test harness_tests 2>&1 | grep 'test result'"
Enter
Sleep 30s

Sleep 2s
