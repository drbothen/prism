Output docs/demo-evidence/S-1.12/AC-002-arc-swap-guard-stable-after-swap.gif
Output docs/demo-evidence/S-1.12/AC-002-arc-swap-guard-stable-after-swap.webm

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

Type "# AC-2 error path (DEC-037): Guard holds old snapshot stable after atomic swap"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests test_BC_2_16_006_guard_holds_old_snapshot_after_swap 2>&1"
Enter
Sleep 15s
