Output docs/demo-evidence/S-DEMO-003/AC-013-env-format-grep-gates.gif
Output docs/demo-evidence/S-DEMO-003/AC-013-env-format-grep-gates.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-013: grep gates — zero matches for retired env-var formats
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-013: env-format grep gates (retired formats must not appear)"
Enter
Sleep 500ms

Type "# Gate 1: retired DEMO_ORG_*_SENSORS credential env-var format"
Enter
Sleep 400ms

Type "grep -rE 'DEMO_ORG_[A-Z_]+_SENSORS' scripts/ docs/ && echo FAIL || echo 'Gate 1: PASS (0 matches)'"
Enter
Sleep 3s

Type "# Gate 2: non-prefixed global bearer token exports"
Enter
Sleep 400ms

Type "grep -rE '^(export )?[A-Z]+_BEARER_TOKEN=' scripts/ docs/ && echo FAIL || echo 'Gate 2: PASS (0 matches)'"
Enter
Sleep 2s
