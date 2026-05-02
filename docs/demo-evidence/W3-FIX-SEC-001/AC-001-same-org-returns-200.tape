Output docs/demo-evidence/W3-FIX-SEC-001/AC-001-same-org-returns-200.gif
Output docs/demo-evidence/W3-FIX-SEC-001/AC-001-same-org-returns-200.webm

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

# AC-001: Same-org X-Org-Id with valid token returns HTTP 200
Type "# AC-001: correct X-Org-Id matches instance_org_id — all 4 clones return 200"
Enter
Sleep 500ms
Type "cargo test -p prism-dtu-claroty -p prism-dtu-crowdstrike -p prism-dtu-cyberint -p prism-dtu-armis --features dtu --test x_org_id_auth -- test_AC_001 --test-threads=1"
Enter
Sleep 15s
