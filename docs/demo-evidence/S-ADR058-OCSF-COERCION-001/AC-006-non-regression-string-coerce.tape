# AC-006: Non-regression — existing EC-016-013-004 and EC-016-013-005 tests remain passing
# Tests: test_coerce_value_string_type_normalizes_integer_to_string (EC-016-013-004)
#        test_coerce_value_string_type_preserves_string_username_against_uid_heuristic (EC-016-013-005)
# Coverage: Non-regression — String+Number and String+String(uid path) unchanged

Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-006-non-regression-string-coerce.gif
Output docs/demo-evidence/S-ADR058-OCSF-COERCION-001/AC-006-non-regression-string-coerce.webm

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

# AC-006: LIVE-DRIFT-003 fix tests (EC-016-013-004 and EC-016-013-005) remain passing
# Confirms: String+Number normalization and uid-suffix heuristic are not regressed
# Both tests were added under fix/claroty-live-api-fidelity (commit 3e9825288)
Type "cargo nextest run -p prism-spec-engine -E 'test(test_coerce_value_string_type_normalizes_integer_to_string) + test(test_coerce_value_string_type_preserves_string_username_against_uid_heuristic)' 2>&1 | tail -8"
Enter
Wait+Line /PASS|FAIL|Summary/
Sleep 2s
