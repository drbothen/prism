# AC-008: build_column_array / coerce_value handle Integer column + Value::Object
# RG-010: test_build_column_array_integer_type_object_input_returns_null_and_emits_warning (Path A)
# RG-011: test_coerce_value_integer_type_object_input_returns_err_coercion_warning (Path B)
# Coverage: Path A (LIVE) + Path B (defense-in-depth) — EC-016-013-030 gap closure
# Wire evidence: holdout HS-004 confirmed (transcript not committed per AD-017)
#   "devices_count": null — key present, JSON null
#   WARN column_coercion_failure column=devices_count column_type=integer actual_json_kind=object

Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-008-integer-object-null-both-paths.gif
Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-008-integer-object-null-both-paths.webm

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

# AC-008: Integer+Object gap (EC-016-013-030) — both Path A and Path B
# Path A: explicit Value::Object(_) => None arm in build_column_array Integer block + warn
# Path B: explicit Value::Object(_) => Err(CoercionWarning) in coerce_value Integer branch
# Before fix: both paths silently passed through / returned None without warning
Type "cargo nextest run -p prism-spec-engine -p prism-bin -E 'test(integer_type_object_input)' 2>&1 | tail -10"
Enter
Sleep 200s
Sleep 2s
