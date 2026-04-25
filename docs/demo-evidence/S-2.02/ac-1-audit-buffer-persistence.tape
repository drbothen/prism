Output ac-1-audit-buffer-persistence.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.02-audit-buffer-watchdog"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-1: Audit entries persist in audit_buffer CF (BC-2.15.003)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_003 --nocapture 2>&1 | tail -8"
Enter
Sleep 15s
