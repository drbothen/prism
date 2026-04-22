Output "docs/demo-evidence/S-1.04/AC-8-selector-unknown-err.gif"
Output "docs/demo-evidence/S-1.04/AC-8-selector-unknown-err.webm"

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

Type "# AC-8 / BC-2.02.012 -- select('vendor_x','unknown_type') -> Err(OcsfUnknownEventClass)"
Enter
Sleep 500ms

Type "cargo test -p prism-ocsf test_BC_2_02_012_unknown_pair_returns_err --no-fail-fast 2>&1 | tail -8"
Enter
Sleep 15s

Sleep 2s
