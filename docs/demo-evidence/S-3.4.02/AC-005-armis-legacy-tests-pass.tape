# AC-005 — Armis legacy tests still pass (27/27 original integration tests)
# Traces to: S-3.4.02 AC-005 (legacy tests unbroken by migration)
# Story: S-3.4.02
#
# VHS tape — produces AC-005-armis-legacy-tests-pass.{gif,webm}

Output docs/demo-evidence/S-3.4.02/AC-005-armis-legacy-tests-pass.gif
Output docs/demo-evidence/S-3.4.02/AC-005-armis-legacy-tests-pass.webm

Require bash

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 60ms

# Setup — hidden
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.02"
Enter
Sleep 1s
Show

# Demo: legacy armis tests (all test suites except harness_tests); all pass, zero failures
Type "bash docs/demo-evidence/S-3.4.02/ac-005-run.sh"
Enter
Sleep 60s

Sleep 2s
