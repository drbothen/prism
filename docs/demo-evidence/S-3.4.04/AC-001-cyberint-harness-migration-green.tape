# AC-001 — Cyberint harness migration tests green
# S-3.4.04: All 26 harness_tests pass against CyberintClone via HarnessBuilder
# Traces to BC-3.5.001 postcondition 1

Output docs/demo-evidence/S-3.4.04/AC-001-cyberint-harness-migration-green.gif
Output docs/demo-evidence/S-3.4.04/AC-001-cyberint-harness-migration-green.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 20
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-3.4.04"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 1s
Show

# Success path — 26 harness_tests all green (BC-3.5.001 postcondition 1)
Type "cargo test -p prism-dtu-cyberint --features dtu --test harness_tests"
Enter
Sleep 10s
Sleep 2s
