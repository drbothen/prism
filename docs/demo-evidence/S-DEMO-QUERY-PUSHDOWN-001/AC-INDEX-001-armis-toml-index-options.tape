Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-INDEX-001-armis-toml-index-options.gif
Output docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/AC-INDEX-001-armis-toml-index-options.webm

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

Type "# AC-INDEX-001: armis.sensor.toml last_seen + created_at declare options=[INDEX]"
Enter
Sleep 500ms

Type "cargo nextest run -p prism-spec-engine -E 'test(ac_index)' --no-fail-fast"
Enter
Wait+Screen@120s /Summary/

Sleep 2s
