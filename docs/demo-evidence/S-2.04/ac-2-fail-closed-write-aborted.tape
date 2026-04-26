Output ac-2-fail-closed-write-aborted.gif

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

Type "# AC-2: Write tool emit() failure -> write aborted, E-AUDIT-001 returned (BC-2.05.001)"
Enter
Sleep 500ms
Type "cargo test -p prism-audit 'tests::bc_2_05_001::test_BC_2_05_001_write_tool_emit_failure' -- --nocapture 2>&1 | grep -E 'test_BC_2_05_001_write|test result'"
Enter
Sleep 15s
