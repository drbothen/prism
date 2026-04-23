Output "docs/demo-evidence/S-1.04/AC-6-selector-crowdstrike.gif"
Output "docs/demo-evidence/S-1.04/AC-6-selector-crowdstrike.webm"

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

Type "# AC-6 / BC-2.02.012 -- select('crowdstrike','detection') -> Ok(2004), not deprecated 2001"
Enter
Sleep 500ms

Type "cargo test -p prism-ocsf test_BC_2_02_012_crowdstrike_detection_returns_2004 test_BC_2_02_012_invariant_no_deprecated_2001_in_any_mapping --no-fail-fast 2>&1 | tail -8"
Enter
Sleep 15s

Sleep 2s
