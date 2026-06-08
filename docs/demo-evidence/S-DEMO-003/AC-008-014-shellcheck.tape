Output docs/demo-evidence/S-DEMO-003/AC-008-014-shellcheck.gif
Output docs/demo-evidence/S-DEMO-003/AC-008-014-shellcheck.webm

Require shellcheck

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-008 + AC-014: shellcheck passes on all demo scripts with zero errors
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-008: local shellcheck gate (zero errors/warnings)"
Enter
Sleep 400ms

Type "shellcheck scripts/demo-setup.sh scripts/demo-run.sh scripts/demo-teardown.sh && echo 'shellcheck: PASS (0 errors, 0 warnings)'"
Enter
Sleep 4s

Type "# AC-014: shellcheck step in ci.yml (HIGH-2)"
Enter
Sleep 400ms

Type "grep -A3 'Run shellcheck on demo scripts' .github/workflows/ci.yml | head -5"
Enter
Sleep 2s
