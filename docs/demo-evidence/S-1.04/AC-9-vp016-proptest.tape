Output "docs/demo-evidence/S-1.04/AC-9-vp016-proptest.gif"
Output "docs/demo-evidence/S-1.04/AC-9-vp016-proptest.webm"

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

Type "# AC-9 / VP-016 -- proptest: all Ok normalize outputs are valid protobuf round-trips"
Enter
Sleep 500ms

Type "cargo test -p prism-ocsf proptest_normalizer --no-fail-fast 2>&1 | tail -8"
Enter
Sleep 20s

Sleep 2s
