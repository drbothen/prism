# AC-001 — CrowdStrike harness migration tests green (47 passed, 2 ignored)
# Traces to: BC-3.5.001 postcondition 1, VP-122
# Story: S-3.4.03
#
# VHS tape — produces AC-001-harness-migration-tests-green.{gif,webm}

Output docs/demo-evidence/S-3.4.03/AC-001-harness-migration-tests-green.gif
Output docs/demo-evidence/S-3.4.03/AC-001-harness-migration-tests-green.webm

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

# Demo — run harness migration tests; expect 47 passed, 2 ignored, 0 failed
Type "cargo test -p prism-dtu-crowdstrike --features dtu --test harness_tests 2>&1 | tail -6"
Enter
Sleep 30s

Sleep 2s
