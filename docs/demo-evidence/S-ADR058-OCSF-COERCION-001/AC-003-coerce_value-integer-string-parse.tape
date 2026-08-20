# AC-003: coerce_value Integer+String — parseable→Ok(Number), non-parseable→Err
# RG-003: test_coerce_value_integer_type_string_non_numeric_path_parse_success_returns_number
# RG-004: test_coerce_value_integer_type_string_non_numeric_path_parse_failure_returns_err
# Coverage: Path B (defense-in-depth) — coerce_value Integer branch in prism-spec-engine

Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-003-coerce_value-integer-string-parse.gif
Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-003-coerce_value-integer-string-parse.webm

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
Wait+Line /\$/
Type "clear"
Enter
Wait+Line /\$/
Show

# AC-003: Integer+String — parse "42" succeeds, parse "not-a-number" fails with Err
# Before fix: both returned Ok(String) pass-through — type mismatch passed silently
# After fix: "42" -> Ok(Number(42)); "not-a-number" -> Err(CoercionWarning)
Type "cargo nextest run -p prism-spec-engine -E 'test(coerce_value_integer_type_string_non_numeric_path)' 2>&1 | tail -10"
Enter
Wait+Line /PASS|FAIL|Summary/
Sleep 2s
