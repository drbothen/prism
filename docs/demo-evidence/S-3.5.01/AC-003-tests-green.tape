# AC-003 — Cargo test suite GREEN (prism-spec-engine post fixture-migration)
# Traces to: BC-3.7.001 postcondition 6 (fixture migration), VP-134
# Story: S-3.5.01
#
# Shows prism-spec-engine tests all passing after fixture path migration from
# tests/fixtures/ to fixtures/.
#
# VHS tape — produces AC-003-tests-green.{gif,webm}

Output docs/demo-evidence/S-3.5.01/AC-003-tests-green.gif
Output docs/demo-evidence/S-3.5.01/AC-003-tests-green.webm

Require bash

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.5.01"
Enter
Sleep 1s
Show

# Demo — run prism-spec-engine tests; tail last 20 lines to see pass summary
Type "cargo test -p prism-spec-engine 2>&1 | tail -20"
Enter
Sleep 30s

Sleep 2s
