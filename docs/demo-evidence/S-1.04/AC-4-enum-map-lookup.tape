Output "docs/demo-evidence/S-1.04/AC-4-enum-map-lookup.gif"
Output "docs/demo-evidence/S-1.04/AC-4-enum-map-lookup.webm"

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 24

Hide
Type "cd /Users/jmagady/dev/prism/.worktrees/S-1.04-ocsf-schema-loading && clear"
Enter
Sleep 1500ms
Show

Type "# AC-4 / BC-2.02.010 -- display_name('severity_id', 4) returns Some('High')"
Enter
Sleep 500ms

Type "cargo test -p prism-ocsf bc_2_02_010 --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 15s

Sleep 2s
