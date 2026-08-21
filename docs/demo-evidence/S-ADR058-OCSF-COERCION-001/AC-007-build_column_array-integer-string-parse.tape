# AC-007: build_column_array ColumnType::Integer arm handles Value::String with parse-attempt
# RG-008: test_build_column_array_integer_type_string_parseable_returns_integer
# RG-009: test_build_column_array_integer_type_string_non_parseable_returns_null_and_emits_warning
# Coverage: Path A (LIVE) — build_column_array Integer arm in prism-bin::spec_driven_adapter
# Wire evidence: holdout HS-002/HS-003 confirmed (transcript not committed per AD-017)
#   "devices_count": null — key present, JSON null
#   WARN column_coercion_failure column=devices_count column_type=integer actual_json_kind=string

Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-007-build_column_array-integer-string-parse.gif
Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-007-build_column_array-integer-string-parse.webm

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

# AC-007: Integer arm — parseable String "42" -> Some(42); non-parseable "not-a-number" -> None+warn
# Before fix: other.as_i64() returned None for all strings silently (no warn, data loss)
# After fix: explicit Value::String(s) arm: parse success->integer; parse fail->null+warn
Type "cargo nextest run -p prism-bin -E 'test(build_column_array_integer_type_string)' 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
