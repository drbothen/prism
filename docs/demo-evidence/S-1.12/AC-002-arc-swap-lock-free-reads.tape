Output docs/demo-evidence/S-1.12/AC-002-arc-swap-lock-free-reads.gif
Output docs/demo-evidence/S-1.12/AC-002-arc-swap-lock-free-reads.webm

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

Type "# AC-2 (BC-2.16.006): 100 concurrent arc-swap loads — no lock contention"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests test_BC_2_16_006_config_load_is_lock_free 2>&1"
Enter
Sleep 15s
