Output docs/demo-evidence/S-1.12/AC-004-add-sensor-spec-invalid-toml.gif
Output docs/demo-evidence/S-1.12/AC-004-add-sensor-spec-invalid-toml.webm

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

Type "# AC-4 error path (EC-002): invalid TOML returns ValidationFailed, no file written"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests test_BC_2_16_008_invalid_toml_returns_validation_failed_no_write 2>&1"
Enter
Sleep 15s
