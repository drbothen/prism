# AC-004 — Harness regression-safe: 70/70 prism-dtu-harness tests still pass
# Traces to: S-3.4.02 AC-004 (no regressions in harness crate itself)
# Story: S-3.4.02
#
# VHS tape — produces AC-004-harness-regression-safe-70.{gif,webm}

Output docs/demo-evidence/S-3.4.02/AC-004-harness-regression-safe-70.gif
Output docs/demo-evidence/S-3.4.02/AC-004-harness-regression-safe-70.webm

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

# Demo: run the full prism-dtu-harness test suite; all 70 pass, zero failures
Type "bash docs/demo-evidence/S-3.4.02/ac-004-run.sh"
Enter
Sleep 60s

Sleep 2s
