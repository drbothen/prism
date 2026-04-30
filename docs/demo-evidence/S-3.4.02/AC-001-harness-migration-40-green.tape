# AC-001 — Armis harness migration: 40/40 harness tests green
# Traces to: S-3.4.02 AC-001, BC-3.5.001
# Story: S-3.4.02
#
# VHS tape — produces AC-001-harness-migration-40-green.{gif,webm}

Output docs/demo-evidence/S-3.4.02/AC-001-harness-migration-40-green.gif
Output docs/demo-evidence/S-3.4.02/AC-001-harness-migration-40-green.webm

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

# Demo: run only the harness_tests binary (40 migrated tests)
Type "bash docs/demo-evidence/S-3.4.02/ac-001-run.sh"
Enter
Sleep 30s

Sleep 2s
