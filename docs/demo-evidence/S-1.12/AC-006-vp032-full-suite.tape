Output docs/demo-evidence/S-1.12/AC-006-vp032-full-suite.gif
Output docs/demo-evidence/S-1.12/AC-006-vp032-full-suite.webm

Require cargo

Set Shell "zsh"
Set FontSize 14
Set FontFamily "FiraCode Nerd Font Mono"
Set Width 1200
Set Height 700
Set Padding 20
Set Theme "Dracula"

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.12-hot-reload"
Enter
Sleep 500ms
Show

Type "# S-1.12 full test suite: 36/37 pass (1 known test-writer bug under triage)"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests 2>&1 | tail -20"
Enter
Sleep 30s
