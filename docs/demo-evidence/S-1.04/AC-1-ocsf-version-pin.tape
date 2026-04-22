Output "docs/demo-evidence/S-1.04/AC-1-ocsf-version-pin.gif"
Output "docs/demo-evidence/S-1.04/AC-1-ocsf-version-pin.webm"

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

Type "# AC-1 / BC-2.02.009 -- ocsf_version() returns compile-time pinned semver"
Enter
Sleep 500ms

Type "cargo test -p prism-ocsf bc_2_02_009 --no-fail-fast 2>&1 | tail -8"
Enter
Sleep 15s

Sleep 2s
