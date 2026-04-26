Output ac-3-structured-json-all-fields.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.04-audit-construction"
Enter
Sleep 2s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-3: Audit entry JSON has all required fields (BC-2.05.002, BC-2.05.008)"
Enter
Sleep 500ms
Type "cargo test -p prism-audit tests::bc_2_05_002 -- --nocapture 2>&1 | grep -E 'test_BC_2_05_002|test result'"
Enter
Sleep 15s
