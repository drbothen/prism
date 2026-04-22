Output docs/demo-evidence/S-1.14/AC-005-hot-reload-atomicity.gif
Output docs/demo-evidence/S-1.14/AC-005-hot-reload-atomicity.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "zsh"
Set WaitTimeout 120s

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.14-infusion-specs"
Enter
Type "clear"
Enter
Show

# AC-5 / BC-2.19.004: hot_reload — failed validation retains previous registry (CI-002)
Type "# AC-5 (BC-2.19.004): hot reload — failed validation retains old registry (CI-002)"
Enter
Sleep 500ms
Type "cargo test -q --package prism-spec-engine BC_2_19_004 2>&1 | grep -E 'ok|FAILED|finished'"
Enter
Wait+Screen /finished/
Sleep 2s
