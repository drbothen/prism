Output "docs/demo-evidence/S-1.04/AC-3-normalize-dynamic-message.gif"
Output "docs/demo-evidence/S-1.04/AC-3-normalize-dynamic-message.webm"

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

Type "# AC-3 / BC-2.02.002 -- normalize(crowdstrike, detection, json) -> DynamicMessage"
Enter
Sleep 500ms

Type "cargo test -p prism-ocsf bc_2_02_002 --no-fail-fast 2>&1 | tail -10"
Enter
Sleep 15s

Sleep 2s
