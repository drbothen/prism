Output ac-6-append-only-immutable.gif

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

Type "# AC-6: Append-only keys, no remove() on AuditBuffer, monotonic ordering (BC-2.05.006)"
Enter
Sleep 500ms
Type "cargo test -p prism-audit tests::bc_2_05_006 -- --nocapture 2>&1 | grep -E 'test_BC_2_05_006|test result'"
Enter
Sleep 15s
