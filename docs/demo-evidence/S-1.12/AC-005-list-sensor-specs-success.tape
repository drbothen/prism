Output docs/demo-evidence/S-1.12/AC-005-list-sensor-specs-success.gif
Output docs/demo-evidence/S-1.12/AC-005-list-sensor-specs-success.webm

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

Type "# AC-5 (BC-2.16.010): list_sensor_specs returns all specs with tables and status"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests test_BC_2_16_010_returns_all_loaded_specs_with_tables_and_status 2>&1"
Enter
Sleep 15s
