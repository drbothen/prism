Output docs/demo-evidence/S-1.12/AC-001-reload-config-applies-new-config.gif
Output docs/demo-evidence/S-1.12/AC-001-reload-config-applies-new-config.webm

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

Type "# AC-1 (BC-2.16.005): reload_config applies new config on success"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests test_BC_2_16_005_reload_applies_new_config_on_success 2>&1"
Enter
Sleep 15s
