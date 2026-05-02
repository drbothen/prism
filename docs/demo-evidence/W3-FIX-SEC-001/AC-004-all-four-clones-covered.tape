Output docs/demo-evidence/W3-FIX-SEC-001/AC-004-all-four-clones-covered.gif
Output docs/demo-evidence/W3-FIX-SEC-001/AC-004-all-four-clones-covered.webm

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

# AC-004: All four DTU clones have full x_org_id_auth test suites
Type "# AC-004: all 4 DTU clones — complete x_org_id_auth suite (30 tests)"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-cyberint -p prism-dtu-armis --features dtu --test x_org_id_auth --test-threads=1"
Enter
Sleep 25s
