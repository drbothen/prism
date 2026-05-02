Output docs/demo-evidence/W3-FIX-SEC-001/AC-005-regression-cross-org-rejected.gif
Output docs/demo-evidence/W3-FIX-SEC-001/AC-005-regression-cross-org-rejected.webm

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

# AC-005: test_cross_org_header_rejected regression in all 4 crates (BC-3.5.002 prec. 3)
Type "# AC-005: test_cross_org_header_rejected regression — BC-3.5.002 prec. 3 (HS-003-02)"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-cyberint -p prism-dtu-armis --features dtu --test x_org_id_auth -- test_cross_org_header_rejected --test-threads=1"
Enter
Sleep 15s
