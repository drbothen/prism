Output docs/demo-evidence/S-1.05/AC-006-armis-timestamp-fallback.gif
Output docs/demo-evidence/S-1.05/AC-006-armis-timestamp-fallback.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Catppuccin Mocha"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

# AC-6 (BC-2.02.006): Armis device with no timestamp fields → valid time (current time fallback).
# Also covers: last_seen priority and created_at fallback.

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-1.05-ocsf-field-mapping"
Enter
Sleep 500ms
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-6: Armis device with no timestamp → current-time fallback (never fails)"
Enter
Sleep 800ms
Type "cargo test -p prism-ocsf test_BC_2_02_006_armis_no_timestamp_falls_back_to_current_time test_BC_2_02_006_armis_last_seen_used_when_present test_BC_2_02_006_armis_created_at_fallback_when_no_last_seen test_BC_2_02_006_armis_mapper_no_timestamp_does_not_fail 2>&1 | grep -E 'test.*ok|test.*FAILED|test result'"
Enter
Sleep 15s
