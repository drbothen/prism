# ALL ACs — Full trailing-slash parity test suite (11 tests, all passing)
#
# Runs every trailing_slash + BC_2_16_013 test in prism-dtu-claroty:
#   AC-001: test_claroty_trailing_slash_alerts_returns_200
#   AC-002: test_claroty_trailing_slash_devices_returns_200
#   AC-003: test_claroty_trailing_slash_audit_log_get_returns_200
#   AC-005/EC-001: test_BC_2_16_013_no_slash_alerts_still_returns_200
#   AC-005/EC-001: test_BC_2_16_013_no_slash_devices_still_returns_200
#   AC-005 intentional: test_BC_2_16_013_tags_route_with_slash_still_works
#   EC-003: test_BC_2_16_013_dtu_health_trailing_slash_returns_200
#   EC-002: test_BC_2_16_013_trailing_slash_alerts_missing_auth_returns_401
#   + 3 audit_log unit tests covering BC-2.16.013
#
# Traces to: BC-2.16.013 v1.25 §Postconditions §1

Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/ALL-ACs-full-suite.gif"
Output "docs/demo-evidence/S-DEMO-CLAROTY-TRAILING-SLASH-001/ALL-ACs-full-suite.webm"

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Catppuccin Mocha"
Set Padding 24
Set TypingSpeed 40ms

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-CLAROTY-TRAILING-SLASH-001"
Enter
Sleep 1s
Show

Type "# S-DEMO-CLAROTY-TRAILING-SLASH-001: full trailing-slash parity suite"
Enter
Sleep 400ms

Type "cargo nextest run -p prism-dtu-claroty --features dtu -E 'test(trailing_slash) | test(BC_2_16_013)'"
Enter
Sleep 30s

Sleep 2s
