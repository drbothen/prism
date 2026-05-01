# AC-006 — CrowdStrike legacy tests still pass (56 legacy + 49 harness = 105 total)
# Traces to: BC-3.5.001 postcondition 1, VP-127
# Story: S-3.4.03
#
# VHS tape — produces AC-006-crowdstrike-legacy-tests-pass.{gif,webm}

Output docs/demo-evidence/S-3.4.03/AC-006-crowdstrike-legacy-tests-pass.gif
Output docs/demo-evidence/S-3.4.03/AC-006-crowdstrike-legacy-tests-pass.webm

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

# Demo — all 105 prism-dtu-crowdstrike tests pass; 0 failures across legacy + harness suites
Type "cargo test -p prism-dtu-crowdstrike --features dtu 2>&1 | grep 'test result' | grep -v '^test result: ok. 0'"
Enter
Sleep 30s

Sleep 2s
