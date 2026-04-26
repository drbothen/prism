Output ac-1-credential-access-event.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.05-audit-events"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-1: credential_name recorded, no value field in JSON (BC-2.05.005)"
Enter
Sleep 500ms
Type "cargo test -p prism-audit BC_2_05_005 -- --nocapture 2>&1 | grep -E 'test_BC_2_05_005|test result'"
Enter
Sleep 20s
