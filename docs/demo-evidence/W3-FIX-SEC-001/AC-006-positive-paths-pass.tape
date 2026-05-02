Output docs/demo-evidence/W3-FIX-SEC-001/AC-006-positive-paths-pass.gif
Output docs/demo-evidence/W3-FIX-SEC-001/AC-006-positive-paths-pass.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-001 && clear"
Enter
Sleep 3s
Show

# AC-006: Pre-existing multi_tenant tests pass without modification (no regression)
Type "# AC-006: pre-existing multi_tenant tests — all pass, no regressions"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-cyberint -p prism-dtu-armis --features dtu --test multi_tenant --test-threads=1"
Enter
Sleep 25s
