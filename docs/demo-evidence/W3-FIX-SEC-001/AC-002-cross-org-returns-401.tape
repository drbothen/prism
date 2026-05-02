Output docs/demo-evidence/W3-FIX-SEC-001/AC-002-cross-org-returns-401.gif
Output docs/demo-evidence/W3-FIX-SEC-001/AC-002-cross-org-returns-401.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cd /Users/jmagady/Dev/prism/.worktrees/W3-FIX-SEC-001 && clear"
Enter
Sleep 3s
Show

# AC-002: Cross-org spoofing returns 401 with JSON error body
Type "# AC-002: cross-org X-Org-Id mismatch — 401 + JSON error body"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-cyberint -p prism-dtu-armis --features dtu --test x_org_id_auth -- test_AC_002 --test-threads=1"
Enter
Sleep 15s
