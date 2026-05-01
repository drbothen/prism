# AC-006 — Cyberint legacy tests still pass (93 total)
# S-3.4.04: All 93 prism-dtu-cyberint tests pass (26 harness + 67 legacy)
# Verifies migration did not regress original AC coverage

Output docs/demo-evidence/S-3.4.04/AC-006-cyberint-legacy-tests-pass.gif
Output docs/demo-evidence/S-3.4.04/AC-006-cyberint-legacy-tests-pass.webm

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

# Success path — all 93 prism-dtu-cyberint tests pass (26 harness + 67 legacy)
Type "cargo test -p prism-dtu-cyberint --features dtu"
Enter
Sleep 20s
Sleep 2s
