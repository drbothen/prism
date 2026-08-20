Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-001-coerce_value-string-array-err.gif
Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-001-coerce_value-string-array-err.webm
Require cargo
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-ADR058-OCSF-COERCION-001 && clear"
Enter
Sleep 0.5s
Show
Type "# AC-001 RG-001: coerce_value String+Array -> Err(CoercionWarning)"
Enter
Sleep 0.3s
Type "cargo nextest run -p prism-spec-engine -E 'test(test_coerce_value_string_type_array_input_returns_err_coercion_warning)' 2>&1 | tail -8"
Enter
Wait+Line /Summary/
Sleep 2s
