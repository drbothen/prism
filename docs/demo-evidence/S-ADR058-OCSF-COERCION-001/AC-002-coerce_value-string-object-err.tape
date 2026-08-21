# AC-002: coerce_value returns Err(CoercionWarning) for String column + Object input
# RG-002: test_coerce_value_string_type_object_input_returns_err_coercion_warning
# Coverage: Path B (defense-in-depth) — coerce_value in prism-spec-engine

Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-002-coerce_value-string-object-err.gif
Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-002-coerce_value-string-object-err.webm

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
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-ADR058-OCSF-COERCION-001"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 2s
Show

# AC-002: String column + Object input must return Err(CoercionWarning)
# Before fix: returned Ok(Object) pass-through — serialized to "{...}" string, wrong data
# After fix: returns Err(CoercionWarning) — value diverted to raw_extensions
Type "cargo nextest run -p prism-spec-engine -E 'test(test_coerce_value_string_type_object_input_returns_err_coercion_warning)' 2>&1 | tail -8"
Enter
Sleep 15s
Sleep 2s
