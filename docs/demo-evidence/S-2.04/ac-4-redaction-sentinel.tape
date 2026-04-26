Output ac-4-redaction-sentinel.gif

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

Type "# AC-4: Credential values replaced with [REDACTED] sentinel (BC-2.05.003)"
Enter
Sleep 500ms
Type "cargo test -p prism-audit tests::bc_2_05_003 -- --nocapture 2>&1 | grep -E 'test_BC_2_05_003|test result'"
Enter
Sleep 15s
