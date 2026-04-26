Output ac-8-prism-audit-requires-audit-read.gif

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-2.03-decorators-internal-tables"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# AC-8: get_descriptor(prism_audit).requires_audit_read == true (BC-2.15.011 E-QUERY-011)"
Enter
Type "cargo test -p prism-storage --lib -- test_BC_2_15_011_get_descriptor_prism_audit_requires_audit_read --nocapture 2>&1 | tail -8"
Enter
Sleep 20s
Sleep 2s
