Output docs/demo-evidence/S-1.12/AC-003-process-spec-changes-validation-failure.gif
Output docs/demo-evidence/S-1.12/AC-003-process-spec-changes-validation-failure.webm

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

Type "# AC-3 error path (EC-003): invalid modified spec retains previous registration"
Enter
Sleep 500ms

Type "cargo test --package prism-spec-engine --test hot_reload_tests test_BC_2_16_007_modified_spec_validation_failure_retains_previous_version 2>&1"
Enter
Sleep 15s
