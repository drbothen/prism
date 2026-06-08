Output docs/demo-evidence/S-DEMO-003/AC-003-004-006-runbook.gif
Output docs/demo-evidence/S-DEMO-003/AC-003-004-006-runbook.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

# AC-003: demo-run.sh structure (DTU launch + overlay generation)
# AC-004: DEMO-RUNBOOK.md §4 Connecting Claude Code present
# AC-006: Troubleshooting section covers 4 failure modes
Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-003"
Enter
Sleep 200ms
Type "clear"
Enter
Sleep 200ms
Show

Type "# AC-003: demo-run.sh shows DTU start + overlay generation structure"
Enter
Sleep 400ms

Type "grep -n overlay scripts/demo-run.sh | head -8"
Enter
Sleep 3s

Type "# AC-004: DEMO-RUNBOOK.md section 4 Connecting Claude Code"
Enter
Sleep 400ms

Type "grep -n 'Connecting Claude Code' docs/DEMO-RUNBOOK.md | head -4"
Enter
Sleep 3s

Type "# AC-006: Troubleshooting — 4 failure modes"
Enter
Sleep 400ms

Type "grep -n 'E-CRED-008' docs/DEMO-RUNBOOK.md | head -6"
Enter
Sleep 2s
