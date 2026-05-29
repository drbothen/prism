Output AC-006-graceful-shutdown.gif
Output AC-006-graceful-shutdown.webm

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

Type "# AC-6: SIGTERM/SIGINT → serve_stdio exits cleanly without buffer corruption"
Enter
Sleep 500ms

Type "# BC-2.10.010: graceful shutdown; 5 shutdown path tests (signal, natural, timeout)"
Enter
Sleep 300ms

Type "cargo nextest run -p prism-mcp -E 'test(shutdown)' --no-capture 2>&1 | tail -30"
Enter
Sleep 12s

Type "# All 5 paths pass: natural close, signal drain, timeout, join_error, complete path"
Enter
Sleep 2s
