Output docs/demo-evidence/S-1.12/AC-004-add-sensor-spec-success.gif
Output docs/demo-evidence/S-1.12/AC-004-add-sensor-spec-success.webm

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

Type "# AC-4 (BC-2.16.008): valid TOML spec upload writes file and registers tables"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests test_BC_2_16_008_valid_new_spec_is_written_and_registered 2>&1"
Enter
Sleep 15s
