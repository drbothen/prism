Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/SEC-004-armis-dtu-security-hardening.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/SEC-004-armis-dtu-security-hardening.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 24
Set Shell "bash"

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/S-DEMO-QUERY-PUSHDOWN-001"
Enter
Sleep 500ms
Show

Type "# SEC-004 defense-in-depth: prism-dtu-armis over-length AQL token returns None (5 tests)"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-dtu-armis --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
