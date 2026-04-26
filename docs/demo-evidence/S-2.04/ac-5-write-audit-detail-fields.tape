Output ac-5-write-audit-detail-fields.gif

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

Type "# AC-5: WriteAuditDetail has capability_check, risk_tier, execution_outcome (BC-2.05.004)"
Enter
Sleep 500ms
Type "cargo test -p prism-audit tests::bc_2_05_004 -- --nocapture 2>&1 | grep -E 'test_BC_2_05_004|test result'"
Enter
Sleep 15s
