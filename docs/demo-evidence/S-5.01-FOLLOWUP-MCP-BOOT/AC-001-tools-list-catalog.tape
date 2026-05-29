Output AC-001-tools-list-catalog.gif
Output AC-001-tools-list-catalog.webm

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 16

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-5.01-FOLLOWUP-MCP-BOOT"
Enter
Sleep 500ms
Show

Type "# AC-1: tools/list returns full canonical tool catalog with DATA TRUST LEVEL + SECURITY NOTE"
Enter
Sleep 500ms

Type "# Run test: BC-2.09.006 + BC-2.10.002 — tool descriptions contain security warnings"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(BC_2_09_006_tool_descriptions_contain_security_warnings)' --no-capture 2>&1 | tail -20"
Enter
Sleep 8s

Type "# Test verifies all sensor tools include DATA TRUST LEVEL + SECURITY NOTE sections"
Enter
Sleep 2s
