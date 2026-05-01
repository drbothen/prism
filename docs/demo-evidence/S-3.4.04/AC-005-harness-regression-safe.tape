# AC-005 — Harness regression-safe (70/70)
# S-3.4.04: All 70 prism-dtu-harness integration tests still pass post-migration
# Verifies no regression in harness itself from cyberint migration

Output docs/demo-evidence/S-3.4.04/AC-005-harness-regression-safe.gif
Output docs/demo-evidence/S-3.4.04/AC-005-harness-regression-safe.webm

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

# Success path — 70 harness integration tests all green (no regression)
Type "cargo test -p prism-dtu-harness --features dtu"
Enter
Sleep 15s
Sleep 2s
