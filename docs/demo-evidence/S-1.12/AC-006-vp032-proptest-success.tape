Output docs/demo-evidence/S-1.12/AC-006-vp032-proptest-success.gif
Output docs/demo-evidence/S-1.12/AC-006-vp032-proptest-success.webm

Require cargo

Set Shell "zsh"
Set FontSize 14
Set FontFamily "FiraCode Nerd Font Mono"
Set Width 1200
Set Height 600
Set Padding 20
Set Theme "Dracula"

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.12-hot-reload"
Enter
Sleep 500ms
Show

Type "# AC-6 (VP-032): proptest — failed validation retains old config (10,000 cases)"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests test_VP_032 2>&1"
Enter
Sleep 30s
